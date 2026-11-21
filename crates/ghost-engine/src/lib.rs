use ghost_dsp::C32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SampleType {
    Real,
    Complex,
}

#[derive(Debug)]
pub struct Node {
    pub sample_type: SampleType,
    pub kind: NodeKind,
}

#[derive(Debug)]
pub enum NodeKind {
    ToneGenerator {
        amplitude: f32,
        frequency: f32,
        phase: f32,
    },
    Sum {
        inputs: usize,
    },
    NullSink,
}

impl Node {
    pub fn input_count(&self) -> usize {
        match self.kind {
            NodeKind::ToneGenerator { .. } => 0,
            NodeKind::Sum { inputs } => inputs,
            NodeKind::NullSink => 1,
        }
    }

    pub fn output_count(&self) -> usize {
        match self.kind {
            NodeKind::ToneGenerator { .. } => 1,
            NodeKind::Sum { .. } => 1,
            NodeKind::NullSink => 0,
        }
    }
}

#[derive(Debug)]
pub struct Graph {
    pub nodes: Vec<Node>,
    pub connections: Vec<Connection>,
}

#[derive(Clone, Copy)]
pub struct SignalBlock<'a, T> {
    pub samples: &'a [T],
    pub start_sample: u64,
}

#[derive(Clone, Copy, Debug)]
pub struct Connection {
    pub source_node: usize,
    pub source_output: usize,
    pub target_node: usize,
    pub target_input: usize,
}

pub struct GraphExecutor {
    pub graph: Graph,
    pub buffers: Vec<Vec<C32>>,
}

impl Graph {
    pub fn add_node(&mut self, node: Node) -> Result<usize, &'static str> {
        match &node.kind {
            NodeKind::Sum { inputs } if *inputs < 2 => {
                return Err("Sum requires at least two inputs");
            }
            _ => {}
        }

        let index = self.nodes.len();
        self.nodes.push(node);
        Ok(index)
    }

    pub fn add_connection(&mut self, connection: Connection) -> Result<(), &'static str> {
        let source = self
            .nodes
            .get(connection.source_node)
            .ok_or("Source node does not exist")?;

        let target = self
            .nodes
            .get(connection.target_node)
            .ok_or("Target node does not exist")?;

        if connection.source_output >= source.output_count() {
            return Err("Invalid source output");
        }

        if connection.target_input >= target.input_count() {
            return Err("Invalid target input");
        }

        if source.sample_type != target.sample_type {
            return Err("Incompatible sample types");
        }

        if self.connections.iter().any(|existing| {
            existing.target_node == connection.target_node
                && existing.target_input == connection.target_input
        }) {
            return Err("Target input is already connected");
        }

        self.connections.push(connection);
        Ok(())
    }
}

impl GraphExecutor {
    pub fn new(graph: Graph, block_size: usize) -> Self {
        let output_count: usize = graph.nodes.iter().map(Node::output_count).sum();

        let buffers = vec![vec![C32::ZERO; block_size]; output_count];

        Self { graph, buffers }
    }

    pub fn input_buffer_index(&self, node_index: usize, port: usize) -> Option<usize> {
        let node = self.graph.nodes.get(node_index)?;

        if port >= node.input_count() {
            return None;
        }

        let connection = self.graph.connections.iter().find(|connection| {
            connection.target_node == node_index && connection.target_input == port
        })?;

        self.output_buffer_index(connection.source_node, connection.source_output)
    }

    pub fn output_buffer_index(&self, node_index: usize, port: usize) -> Option<usize> {
        let node = self.graph.nodes.get(node_index)?;

        if port >= node.output_count() {
            return None;
        }

        let offset: usize = self.graph.nodes[..node_index]
            .iter()
            .map(Node::output_count)
            .sum();

        Some(offset + port)
    }

    pub fn process_node(
        &mut self,
        node_index: usize,
        sample_rate: f32,
    ) -> Result<(), &'static str> {
        let buffer_index = self
            .output_buffer_index(node_index, 0)
            .ok_or("Node or output port not found")?;

        let node = &mut self.graph.nodes[node_index];

        match (&mut node.kind, node.sample_type) {
            (
                NodeKind::ToneGenerator {
                    amplitude,
                    frequency,
                    phase,
                },
                SampleType::Complex,
            ) => {
                *phase = ghost_dsp::generate_complex_tone(
                    sample_rate,
                    *amplitude,
                    *frequency,
                    *phase,
                    &mut self.buffers[buffer_index],
                );
                Ok(())
            }
            _ => Err("Node execution not implemented"),
        }
    }
}
