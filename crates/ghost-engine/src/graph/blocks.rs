use std::sync::{Arc, Mutex, MutexGuard};

use ghost_dsp::{Complex32, ComplexOscillator};

use super::block::{Block, BlockIo, PortType};

pub struct SignalGenerator {
    oscillator: ComplexOscillator,
    remaining: Option<u64>,
}

impl SignalGenerator {
    pub fn new(oscillator: ComplexOscillator, total_samples: Option<u64>) -> Self {
        Self {
            oscillator,
            remaining: total_samples,
        }
    }

    pub fn oscillator_mut(&mut self) -> &mut ComplexOscillator {
        &mut self.oscillator
    }
}

impl Block for SignalGenerator {
    fn output_ports(&self) -> &[PortType] {
        &[PortType::Complex32]
    }

    fn process(&mut self, io: &mut BlockIo) {
        let out = io.output::<Complex32>(0);
        let count = match self.remaining {
            Some(remaining) => out
                .len()
                .min(usize::try_from(remaining).unwrap_or(usize::MAX)),
            None => out.len(),
        };
        if count > 0 {
            self.oscillator.fill(&mut out[..count]);
            io.produce(0, count);
        }
        if let Some(remaining) = self.remaining.as_mut() {
            *remaining -= count as u64;
            if *remaining == 0 {
                io.finish();
            }
        }
    }
}

pub struct Gain {
    gain: f32,
}

impl Gain {
    pub fn new(gain: f32) -> Self {
        Self { gain }
    }

    pub fn gain(&self) -> f32 {
        self.gain
    }

    pub fn set_gain(&mut self, gain: f32) {
        self.gain = gain;
    }
}

impl Block for Gain {
    fn input_ports(&self) -> &[PortType] {
        &[PortType::Complex32]
    }

    fn output_ports(&self) -> &[PortType] {
        &[PortType::Complex32]
    }

    fn process(&mut self, io: &mut BlockIo) {
        let gain = self.gain;
        let (input, output) = io.input_output::<Complex32, Complex32>(0, 0);
        let count = input.len().min(output.len());
        for (out, sample) in output[..count].iter_mut().zip(&input[..count]) {
            *out = *sample * gain;
        }
        if count > 0 {
            io.consume(0, count);
            io.produce(0, count);
        }
        if io.input_exhausted(0) {
            io.finish();
        }
    }
}

#[derive(Clone, Default)]
pub struct CollectHandle {
    samples: Arc<Mutex<Vec<Complex32>>>,
}

impl CollectHandle {
    fn lock(&self) -> MutexGuard<'_, Vec<Complex32>> {
        self.samples
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub fn len(&self) -> usize {
        self.lock().len()
    }

    pub fn is_empty(&self) -> bool {
        self.lock().is_empty()
    }

    pub fn samples(&self) -> Vec<Complex32> {
        self.lock().clone()
    }
}

pub struct CollectSink {
    handle: CollectHandle,
}

impl CollectSink {
    pub fn new(expected_samples: usize) -> (Self, CollectHandle) {
        let handle = CollectHandle {
            samples: Arc::new(Mutex::new(Vec::with_capacity(expected_samples))),
        };
        (
            Self {
                handle: handle.clone(),
            },
            handle,
        )
    }
}

impl Block for CollectSink {
    fn input_ports(&self) -> &[PortType] {
        &[PortType::Complex32]
    }

    fn process(&mut self, io: &mut BlockIo) {
        let input = io.input::<Complex32>(0);
        let count = input.len();
        if count > 0 {
            self.handle.lock().extend_from_slice(input);
            io.consume(0, count);
        }
        if io.input_exhausted(0) {
            io.finish();
        }
    }
}
