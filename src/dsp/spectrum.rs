use std::sync::Arc;

use realfft::num_complex::Complex32;
use realfft::{RealFftPlanner, RealToComplex};

pub struct Spectrum {
    fft: Arc<dyn RealToComplex<f32>>,
    input: Vec<f32>,
    output: Vec<Complex32>,
    scratch: Vec<Complex32>,
}

impl Spectrum {
    pub fn new(size: usize) -> Self {
        let mut planner = RealFftPlanner::<f32>::new();
        let fft = planner.plan_fft_forward(size);

        Self {
            input: fft.make_input_vec(),
            output: fft.make_output_vec(),
            scratch: fft.make_scratch_vec(),
            fft,
        }
    }

    pub fn process(&mut self, samples: &[f32]) -> impl Iterator<Item = f32> + '_ {
        self.input.copy_from_slice(samples);

        self.fft
            .process_with_scratch(&mut self.input, &mut self.output, &mut self.scratch)
            .unwrap();

        let size = self.input.len();
        let scale = 1.0 / size as f32;

        self.output.iter().enumerate().map(move |(i, bin)| {
            if i == 0 || (size.is_multiple_of(2) && i == size / 2) {
                bin.norm() * scale
            } else {
                bin.norm() * scale * 2.0
            }
        })
    }
}
