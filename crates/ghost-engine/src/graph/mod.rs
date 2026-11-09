mod block;
mod blocks;
mod ring;
mod scheduler;

use std::fmt;

pub use block::{Block, BlockIo, PortType, Sample};
pub use blocks::{CollectHandle, CollectSink, Gain, SignalGenerator};
pub use ring::RingBuffer;
pub use scheduler::{Scheduler, Status};

use block::{Buffer, Edge};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockId(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortDirection {
    Input,
    Output,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraphError {
    UnknownBlock(BlockId),
    InvalidPort {
        block: BlockId,
        direction: PortDirection,
        port: usize,
    },
    PortAlreadyConnected {
        block: BlockId,
        direction: PortDirection,
        port: usize,
    },
    PortTypeMismatch {
        output: PortType,
        input: PortType,
    },
    ZeroCapacity,
    Cycle,
    UnconnectedPort {
        block: BlockId,
        direction: PortDirection,
        port: usize,
    },
    Stalled,
}

impl fmt::Display for GraphError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownBlock(id) => write!(f, "unknown block {}", id.0),
            Self::InvalidPort {
                block,
                direction,
                port,
            } => write!(f, "block {} has no {direction:?} port {port}", block.0),
            Self::PortAlreadyConnected {
                block,
                direction,
                port,
            } => write!(
                f,
                "{direction:?} port {port} of block {} is already connected",
                block.0
            ),
            Self::PortTypeMismatch { output, input } => {
                write!(f, "cannot connect {output:?} output to {input:?} input")
            }
            Self::ZeroCapacity => write!(f, "connection capacity must be positive"),
            Self::Cycle => write!(f, "connection would create a cycle"),
            Self::UnconnectedPort {
                block,
                direction,
                port,
            } => write!(
                f,
                "{direction:?} port {port} of block {} is not connected",
                block.0
            ),
            Self::Stalled => write!(f, "graph made no progress before completing"),
        }
    }
}

impl std::error::Error for GraphError {}

pub(crate) struct Node {
    pub block: Box<dyn Block>,
    pub inputs: Vec<Option<usize>>,
    pub outputs: Vec<Option<usize>>,
}

#[derive(Default)]
pub struct Graph {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
}

impl Graph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_block(&mut self, block: impl Block + 'static) -> BlockId {
        let inputs = vec![None; block.input_ports().len()];
        let outputs = vec![None; block.output_ports().len()];
        self.nodes.push(Node {
            block: Box::new(block),
            inputs,
            outputs,
        });
        BlockId(self.nodes.len() - 1)
    }

    pub fn connect(
        &mut self,
        from: BlockId,
        output_port: usize,
        to: BlockId,
        input_port: usize,
        capacity: usize,
    ) -> Result<(), GraphError> {
        let source = self
            .nodes
            .get(from.0)
            .ok_or(GraphError::UnknownBlock(from))?;
        let target = self.nodes.get(to.0).ok_or(GraphError::UnknownBlock(to))?;
        let output_type =
            *source
                .block
                .output_ports()
                .get(output_port)
                .ok_or(GraphError::InvalidPort {
                    block: from,
                    direction: PortDirection::Output,
                    port: output_port,
                })?;
        let input_type =
            *target
                .block
                .input_ports()
                .get(input_port)
                .ok_or(GraphError::InvalidPort {
                    block: to,
                    direction: PortDirection::Input,
                    port: input_port,
                })?;
        if source.outputs[output_port].is_some() {
            return Err(GraphError::PortAlreadyConnected {
                block: from,
                direction: PortDirection::Output,
                port: output_port,
            });
        }
        if target.inputs[input_port].is_some() {
            return Err(GraphError::PortAlreadyConnected {
                block: to,
                direction: PortDirection::Input,
                port: input_port,
            });
        }
        if output_type != input_type {
            return Err(GraphError::PortTypeMismatch {
                output: output_type,
                input: input_type,
            });
        }
        if capacity == 0 {
            return Err(GraphError::ZeroCapacity);
        }
        if from == to || self.reaches(to.0, from.0) {
            return Err(GraphError::Cycle);
        }
        let index = self.edges.len();
        self.edges.push(Edge {
            buffer: Buffer::new(output_type, capacity),
            closed: false,
            from: from.0,
            to: to.0,
        });
        self.nodes[from.0].outputs[output_port] = Some(index);
        self.nodes[to.0].inputs[input_port] = Some(index);
        Ok(())
    }

    fn reaches(&self, start: usize, target: usize) -> bool {
        let mut visited = vec![false; self.nodes.len()];
        let mut stack = vec![start];
        while let Some(node) = stack.pop() {
            if node == target {
                return true;
            }
            if std::mem::replace(&mut visited[node], true) {
                continue;
            }
            stack.extend(self.edges.iter().filter(|e| e.from == node).map(|e| e.to));
        }
        false
    }

    pub(crate) fn into_parts(self) -> (Vec<Node>, Vec<Edge>) {
        (self.nodes, self.edges)
    }
}
