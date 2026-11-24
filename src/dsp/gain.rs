use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

#[derive(Clone)]
pub struct Gain {
    gain: Arc<AtomicU32>,
}

impl Gain {
    pub fn new(gain: f32) -> Self {
        Self {
            gain: Arc::new(AtomicU32::new(gain.to_bits())),
        }
    }

    pub fn set_gain(&self, gain: f32) {
        self.gain.store(gain.to_bits(), Ordering::Relaxed);
    }

    pub fn gain(&self) -> f32 {
        f32::from_bits(self.gain.load(Ordering::Relaxed))
    }

    pub fn process(&self, input: &[f32], output: &mut [f32]) {
        let gain = self.gain();

        for (input, output) in input.iter().zip(output.iter_mut()) {
            *output = *input * gain;
        }
    }
}
