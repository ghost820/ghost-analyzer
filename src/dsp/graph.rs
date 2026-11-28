use super::file_source::FileSource;
use super::gain::Gain;
use super::plot_tap::PlotTap;

const BLOCK_FRAMES: usize = 256;

pub enum RuntimeNode {
    FileSource(FileSource),
    Gain { input: usize, gain: Gain },
}

pub struct RuntimeGraph {
    nodes: Vec<RuntimeNode>,
    buffers: Vec<f32>,
    plot_taps: Vec<(usize, PlotTap)>,
    block_size: usize,
    output_node: usize,
}

impl RuntimeGraph {
    pub fn new(nodes: Vec<RuntimeNode>, output_node: usize, channels: u16) -> Self {
        let block_size = BLOCK_FRAMES * usize::from(channels);
        let buffers = vec![0.0; nodes.len() * block_size];
        let plot_taps = vec![];

        Self {
            nodes,
            buffers,
            plot_taps,
            block_size,
            output_node,
        }
    }

    pub fn add_plot_tap(&mut self, input: usize, tap: PlotTap) {
        self.plot_taps.push((input, tap));
    }

    pub fn process(&mut self, output: &mut [f32]) {
        for chunk in output.chunks_mut(self.block_size) {
            let len = chunk.len();

            self.process_block(len);

            let offset = self.output_node * self.block_size;
            chunk.copy_from_slice(&self.buffers[offset..offset + len]);
        }
    }

    pub fn process_block(&mut self, len: usize) {
        for (index, node) in self.nodes.iter_mut().enumerate() {
            let offset = index * self.block_size;
            let (previous, current) = self.buffers.split_at_mut(offset);
            let target = &mut current[..len];

            match node {
                RuntimeNode::FileSource(source) => {
                    source.process(target);
                }
                RuntimeNode::Gain { input, gain } => {
                    let offset = *input * self.block_size;
                    gain.process(&previous[offset..offset + len], target);
                }
            }
        }

        for (input, tap) in &mut self.plot_taps {
            let offset = *input * self.block_size;
            tap.process(&self.buffers[offset..offset + len]);
        }
    }
}
