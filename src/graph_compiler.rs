use std::collections::HashMap;

use egui_snarl::{InPinId, NodeId, Snarl};

use crate::Node;
use crate::dsp::{self, RuntimeGraph, RuntimeNode};

pub fn compile(graph: &Snarl<Node>) -> (RuntimeGraph, u32, u16) {
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

    (
        RuntimeGraph::new(nodes, output_node, channels),
        sample_rate,
        channels,
    )
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
        Node::AudioSink => unreachable!(),
    };

    let index = nodes.len();

    nodes.push(runtime_node);
    indices.insert(node, index);

    index
}

fn input_node(graph: &Snarl<Node>, node: NodeId) -> NodeId {
    graph.in_pin(InPinId { node, input: 0 }).remotes[0].node
}
