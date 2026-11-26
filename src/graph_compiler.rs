use std::collections::HashMap;

use egui_snarl::{InPinId, NodeId, Snarl};

use crate::Node;
use crate::dsp::{self, PlotReader, PlotTap, RuntimeGraph, RuntimeNode};

pub struct CompiledGraph {
    pub runtime: RuntimeGraph,
    pub sample_rate: u32,
    pub channels: u16,
    pub plot_readers: Vec<(NodeId, PlotReader)>,
}

pub fn compile(graph: &Snarl<Node>) -> CompiledGraph {
    let sink = graph
        .node_ids()
        .find_map(|(id, node)| matches!(node, Node::AudioSink).then_some(id))
        .unwrap();

    let mut nodes = Vec::new();
    let mut indices = HashMap::new();

    let output_node = compile_node(graph, input_node(graph, sink), &mut nodes, &mut indices);

    let RuntimeNode::FileSource(source) = &nodes[0] else {
        unreachable!();
    };

    let sample_rate = source.sample_rate;
    let channels = source.channels;

    let mut plot_taps = Vec::new();
    let mut plot_readers = Vec::new();

    for (id, node) in graph.node_ids() {
        if matches!(node, Node::TimePlot(_) | Node::FftPlot(_)) {
            let input = compile_node(graph, input_node(graph, id), &mut nodes, &mut indices);
            let (tap, reader) = PlotTap::new(channels);

            plot_taps.push((input, tap));
            plot_readers.push((id, reader));
        }
    }

    let mut runtime = RuntimeGraph::new(nodes, output_node, channels);

    for (input, tap) in plot_taps {
        runtime.add_plot_tap(input, tap);
    }

    CompiledGraph {
        runtime,
        sample_rate,
        channels,
        plot_readers,
    }
}

fn compile_node(
    graph: &Snarl<Node>,
    node: NodeId,
    nodes: &mut Vec<RuntimeNode>,
    indices: &mut HashMap<NodeId, usize>,
) -> usize {
    if let Some(&index) = indices.get(&node) {
        return index;
    }

    let runtime_node = match &graph[node] {
        Node::FileSource { path } => RuntimeNode::FileSource(dsp::FileSource::new(path)),
        Node::Gain(dsp) => {
            let input = compile_node(graph, input_node(graph, node), nodes, indices);

            RuntimeNode::Gain {
                input,
                gain: dsp.clone(),
            }
        }
        Node::AudioSink | Node::TimePlot(_) | Node::FftPlot(_) => unreachable!(),
    };

    let index = nodes.len();

    nodes.push(runtime_node);
    indices.insert(node, index);

    index
}

fn input_node(graph: &Snarl<Node>, node: NodeId) -> NodeId {
    graph.in_pin(InPinId { node, input: 0 }).remotes[0].node
}
