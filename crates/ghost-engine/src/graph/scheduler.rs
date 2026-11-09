use super::block::{Block, BlockIo, Edge};
use super::{BlockId, Graph, GraphError, PortDirection};

struct RunNode {
    block: Box<dyn Block>,
    inputs: Vec<usize>,
    outputs: Vec<usize>,
    finished: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Running,
    Complete,
}

pub struct Scheduler {
    nodes: Vec<RunNode>,
    edges: Vec<Edge>,
    order: Vec<usize>,
    remaining: usize,
}

impl Scheduler {
    pub fn new(graph: Graph) -> Result<Self, GraphError> {
        let (nodes, edges) = graph.into_parts();
        let mut run_nodes = Vec::with_capacity(nodes.len());
        for (index, node) in nodes.into_iter().enumerate() {
            let resolve = |ports: &[Option<usize>], direction| {
                ports
                    .iter()
                    .enumerate()
                    .map(|(port, edge)| {
                        edge.ok_or(GraphError::UnconnectedPort {
                            block: BlockId(index),
                            direction,
                            port,
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()
            };
            run_nodes.push(RunNode {
                inputs: resolve(&node.inputs, PortDirection::Input)?,
                outputs: resolve(&node.outputs, PortDirection::Output)?,
                block: node.block,
                finished: false,
            });
        }
        let order = topological_order(run_nodes.len(), &edges)?;
        let remaining = run_nodes.len();
        Ok(Self {
            nodes: run_nodes,
            edges,
            order,
            remaining,
        })
    }

    pub fn is_complete(&self) -> bool {
        self.remaining == 0
    }

    pub fn step(&mut self) -> Result<Status, GraphError> {
        if self.remaining == 0 {
            return Ok(Status::Complete);
        }
        let mut progressed = false;
        for &index in &self.order {
            let node = &mut self.nodes[index];
            if node.finished {
                continue;
            }
            let mut io = BlockIo::new(&mut self.edges, &node.inputs, &node.outputs);
            node.block.process(&mut io);
            progressed |= io.progressed();
            if io.finished() {
                node.finished = true;
                self.remaining -= 1;
            }
        }
        if self.remaining == 0 {
            Ok(Status::Complete)
        } else if progressed {
            Ok(Status::Running)
        } else {
            Err(GraphError::Stalled)
        }
    }

    pub fn run(&mut self) -> Result<(), GraphError> {
        while self.step()? == Status::Running {}
        Ok(())
    }
}

fn topological_order(count: usize, edges: &[Edge]) -> Result<Vec<usize>, GraphError> {
    let mut in_degree = vec![0usize; count];
    for edge in edges {
        in_degree[edge.to] += 1;
    }
    let mut order: Vec<usize> = (0..count).filter(|&n| in_degree[n] == 0).collect();
    let mut cursor = 0;
    while cursor < order.len() {
        let node = order[cursor];
        cursor += 1;
        for edge in edges.iter().filter(|e| e.from == node) {
            in_degree[edge.to] -= 1;
            if in_degree[edge.to] == 0 {
                order.push(edge.to);
            }
        }
    }
    if order.len() == count {
        Ok(order)
    } else {
        Err(GraphError::Cycle)
    }
}
