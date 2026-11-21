#![allow(non_snake_case)]

use std::f32::consts::TAU;
use std::sync::Arc;

pub use rustfft::num_complex::Complex32 as C32;
use rustfft::num_complex::ComplexFloat;

pub const fn c32(re: f32, im: f32) -> C32 {
    C32::new(re, im)
}

pub fn generate_complex_tone(fs: f32, A: f32, freq: f32, mut phase: f32, out: &mut [C32]) -> f32 {
    let phase_step = TAU * freq / fs;

    for sample in out {
        let (q, i) = phase.sin_cos();
        *sample = c32(A * i, A * q);
        phase = (phase + phase_step).rem_euclid(TAU);
    }

    phase
}

pub fn sum_complex(inputs: &[&[C32]], out: &mut [C32]) {
    for input in inputs {
        for (result, &sample) in out.iter_mut().zip(input.iter()) {
            *result += sample;
        }
    }
}

pub fn mean_power(samples: &[C32]) -> f32 {
    let mut sum = 0.0;

    for s in samples {
        sum += s.norm_sqr();
    }

    sum / samples.len() as f32
}

pub fn mean_power_db(samples: &[C32]) -> f32 {
    power_to_db(mean_power(samples))
}

pub fn rms(samples: &[C32]) -> f32 {
    mean_power(samples).sqrt()
}

pub fn mag_to_db(mag: f32) -> f32 {
    20.0 * mag.log10()
}

pub fn power_to_db(power: f32) -> f32 {
    10.0 * power.log10()
}

pub struct Fft {
    fft: Arc<dyn rustfft::Fft<f32>>,
    scratch: Vec<C32>,
}

impl Fft {
    pub fn new(size: usize) -> Self {
        let mut planner = rustfft::FftPlanner::new();
        let fft = planner.plan_fft_forward(size);
        let scratch = vec![C32::ZERO; fft.get_inplace_scratch_len()];

        Self { fft, scratch }
    }

    pub fn process(&mut self, samples: &mut [C32]) {
        self.fft.process_with_scratch(samples, &mut self.scratch);
    }
}

pub struct SpectrumAnalyzer {
    fft: Fft,
    window: Vec<f32>,
    weight_sum: f32,
}

impl SpectrumAnalyzer {
    pub fn new(size: usize) -> Self {
        let fft = Fft::new(size);

        let mut window = vec![0.0; size];
        hann_window(&mut window);

        let weight_sum = window.iter().sum();

        Self {
            fft,
            window,
            weight_sum,
        }
    }

    pub fn process(&mut self, samples: &mut [C32], out: &mut [f32]) {
        apply_window(samples, &self.window);
        self.fft.process(samples);
        fft_magnitude(samples, self.weight_sum, out);
    }
}

pub fn apply_window(samples: &mut [C32], window: &[f32]) {
    for (sample, &w) in samples.iter_mut().zip(window) {
        *sample *= w;
    }
}

pub fn hann_window(out: &mut [f32]) {
    let N = out.len() as f32;

    for (n, w) in out.iter_mut().enumerate() {
        *w = 0.5 - 0.5 * (TAU * n as f32 / N).cos();
    }
}

pub fn fft_magnitude(bins: &[C32], weight_sum: f32, out: &mut [f32]) {
    let scale = 1.0 / weight_sum;

    for (bin, magnitude) in bins.iter().zip(out) {
        *magnitude = bin.abs() * scale;
    }
}

pub fn fft_bin_freq(fs: f32, N: usize, k: usize) -> f32 {
    let index = if k < N.div_ceil(2) {
        k as f32
    } else {
        k as f32 - N as f32
    };

    index * fs / N as f32
}

#[cfg(test)]
mod complex_tone_tests {
    use super::*;

    #[test]
    fn generates_complex_tone() {
        let mut samples = [C32::ZERO; 4];

        generate_complex_tone(4.0, 1.0, 1.0, 0.0, &mut samples);

        let expected = [c32(1.0, 0.0), c32(0.0, 1.0), c32(-1.0, 0.0), c32(0.0, -1.0)];

        for (actual, expected) in samples.iter().zip(expected) {
            assert!((actual - expected).abs() < 1e-6);
        }
    }

    #[test]
    fn applies_amplitude() {
        let mut samples = [C32::ZERO; 4];

        generate_complex_tone(4.0, 0.5, 1.0, 0.0, &mut samples);

        let expected = [c32(0.5, 0.0), c32(0.0, 0.5), c32(-0.5, 0.0), c32(0.0, -0.5)];

        for (actual, expected) in samples.iter().zip(expected) {
            assert!((actual - expected).abs() < 1e-6);
        }
    }

    #[test]
    fn preserves_phase_continuity() {
        let mut full = [C32::ZERO; 8];
        generate_complex_tone(8.0, 1.0, 1.0, 0.0, &mut full);

        let mut first = [C32::ZERO; 3];
        let mut second = [C32::ZERO; 5];

        let phase = generate_complex_tone(8.0, 1.0, 1.0, 0.0, &mut first);
        generate_complex_tone(8.0, 1.0, 1.0, phase, &mut second);

        for (actual, expected) in first.iter().chain(&second).zip(full) {
            assert!((actual - expected).abs() < 1e-6);
        }
    }

    #[test]
    fn generates_negative_frequency() {
        let mut samples = [C32::ZERO; 4];

        generate_complex_tone(4.0, 1.0, -1.0, 0.0, &mut samples);

        let expected = [c32(1.0, 0.0), c32(0.0, -1.0), c32(-1.0, 0.0), c32(0.0, 1.0)];

        for (actual, expected) in samples.iter().zip(expected) {
            assert!((actual - expected).abs() < 1e-6);
        }
    }
}

#[cfg(test)]
mod power_tests {
    use super::*;

    #[test]
    fn calculates_mean_power() {
        let samples = [c32(3.0, 4.0), C32::ZERO];

        let power = mean_power(&samples);

        assert_eq!(power, 12.5);
    }

    #[test]
    fn calculates_mean_power_db() {
        let samples = [c32(0.5, 0.0), c32(0.0, 0.5)];

        let power = mean_power_db(&samples);

        assert!((power + 6.0206).abs() < 1e-4);
    }

    #[test]
    fn calculates_rms() {
        let samples = [c32(3.0, 4.0), C32::ZERO];

        let result = rms(&samples);

        assert!((result - 12.5_f32.sqrt()).abs() < 1e-6);
    }

    #[test]
    fn converts_mag_to_db() {
        assert_eq!(mag_to_db(1.0), 0.0);
        assert!((mag_to_db(0.5) + 6.0206).abs() < 1e-4);
        assert!((mag_to_db(2.0) - 6.0206).abs() < 1e-4);
    }

    #[test]
    fn converts_power_to_db() {
        assert_eq!(power_to_db(1.0), 0.0);
        assert!((power_to_db(0.25) + 6.0206).abs() < 1e-4);
        assert!((power_to_db(4.0) - 6.0206).abs() < 1e-4);
    }
}

#[cfg(test)]
mod fft_tests {
    use super::*;

    #[test]
    fn detects_tone_magnitude() {
        let mut samples = [C32::ZERO; 8];
        generate_complex_tone(8.0, 0.5, 1.0, 0.0, &mut samples);

        let mut fft = Fft::new(8);
        fft.process(&mut samples);

        let mut magnitudes = [0.0; 8];
        fft_magnitude(&samples, samples.len() as f32, &mut magnitudes);

        assert!((magnitudes[1] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn detects_windowed_tone_magnitude() {
        let mut samples = [C32::ZERO; 8];
        let mut window = [0.0; 8];

        generate_complex_tone(8.0, 0.5, 1.0, 0.0, &mut samples);
        hann_window(&mut window);
        apply_window(&mut samples, &window);

        let mut fft = Fft::new(8);
        fft.process(&mut samples);

        let mut magnitudes = [0.0; 8];
        let weight_sum = window.iter().sum();

        fft_magnitude(&samples, weight_sum, &mut magnitudes);

        assert!((magnitudes[1] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn maps_fft_bins_to_frequencies() {
        let expected = [
            0.0, 1000.0, 2000.0, 3000.0, -4000.0, -3000.0, -2000.0, -1000.0,
        ];

        for (k, freq) in expected.into_iter().enumerate() {
            assert_eq!(fft_bin_freq(8000.0, 8, k), freq);
        }
    }

    #[test]
    fn generates_hann_window() {
        let mut window = [0.0; 4];

        hann_window(&mut window);

        let expected = [0.0, 0.5, 1.0, 0.5];

        for (actual, expected) in window.iter().zip(expected) {
            assert!((actual - expected).abs() < 1e-6);
        }
    }

    #[test]
    fn applies_hann_window() {
        let mut samples = [c32(1.0, 1.0); 4];
        let mut window = [0.0; 4];

        hann_window(&mut window);
        apply_window(&mut samples, &window);

        let expected = [C32::ZERO, c32(0.5, 0.5), c32(1.0, 1.0), c32(0.5, 0.5)];

        for (actual, expected) in samples.iter().zip(expected) {
            assert!((actual - expected).abs() < 1e-6);
        }
    }
}
