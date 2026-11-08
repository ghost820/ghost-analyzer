use std::sync::Arc;

use num_complex::Complex32;
use rustfft::{Fft, FftPlanner};

use crate::{window::hann_periodic, DspError};

const MIN_MAGNITUDE: f32 = 1e-9;

pub struct SpectrumAnalyzer {
    size: usize,
    fft: Arc<dyn Fft<f32>>,
    window: Vec<f32>,
    inv_gain: f32,
    buffer: Vec<Complex32>,
    scratch: Vec<Complex32>,
}

impl SpectrumAnalyzer {
    pub fn new(size: usize) -> Result<Self, DspError> {
        if size < 2 || !size.is_power_of_two() {
            return Err(DspError::InvalidFftSize(size));
        }
        let fft = FftPlanner::<f32>::new().plan_fft_forward(size);
        let window = hann_periodic(size);
        let gain: f32 = window.iter().sum();
        let scratch = vec![Complex32::default(); fft.get_inplace_scratch_len()];
        Ok(Self {
            size,
            fft,
            window,
            inv_gain: 1.0 / gain,
            buffer: vec![Complex32::default(); size],
            scratch,
        })
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn process_db(&mut self, input: &[Complex32], out: &mut [f32]) -> Result<(), DspError> {
        if input.len() != self.size {
            return Err(DspError::LengthMismatch {
                expected: self.size,
                actual: input.len(),
            });
        }
        if out.len() != self.size {
            return Err(DspError::LengthMismatch {
                expected: self.size,
                actual: out.len(),
            });
        }

        for ((dst, src), w) in self.buffer.iter_mut().zip(input).zip(&self.window) {
            *dst = src * *w;
        }
        self.fft
            .process_with_scratch(&mut self.buffer, &mut self.scratch);

        let half = self.size / 2;
        let inv_gain = self.inv_gain;
        let to_db = |c: &Complex32| 20.0 * (c.norm() * inv_gain).max(MIN_MAGNITUDE).log10();
        for (dst, bin) in out[..half].iter_mut().zip(&self.buffer[half..]) {
            *dst = to_db(bin);
        }
        for (dst, bin) in out[half..].iter_mut().zip(&self.buffer[..half]) {
            *dst = to_db(bin);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ComplexOscillator;

    fn peak_bin(spectrum: &[f32]) -> usize {
        spectrum
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.total_cmp(b.1))
            .map(|(i, _)| i)
            .unwrap()
    }

    fn analyze(fs: f64, f: f64, a: f64, size: usize) -> Vec<f32> {
        let mut osc = ComplexOscillator::new(fs, f, a).unwrap();
        let mut iq = vec![Complex32::default(); size];
        osc.fill(&mut iq);
        let mut analyzer = SpectrumAnalyzer::new(size).unwrap();
        let mut out = vec![0.0; size];
        analyzer.process_db(&iq, &mut out).unwrap();
        out
    }

    #[test]
    fn peak_lands_on_expected_bin() {
        let size = 1024;
        let fs = 1024.0;
        for k in [-300i32, -1, 0, 1, 64, 250] {
            let spectrum = analyze(fs, f64::from(k), 1.0, size);
            let expected = (size as i32 / 2 + k) as usize;
            assert_eq!(peak_bin(&spectrum), expected, "k={k}");
        }
    }

    #[test]
    fn bin_centered_tone_reads_its_amplitude_in_dbfs() {
        let spectrum = analyze(2048.0, 256.0, 1.0, 2048);
        let peak = spectrum[peak_bin(&spectrum)];
        assert!(peak.abs() < 0.05, "peak={peak}");

        let half = analyze(2048.0, 256.0, 0.5, 2048);
        let peak = half[peak_bin(&half)];
        assert!((peak + 6.0206).abs() < 0.05, "peak={peak}");
    }

    #[test]
    fn off_bin_tone_peak_is_near_true_frequency() {
        let size = 2048;
        let fs = 48_000.0;
        let f = 5_000.0;
        let spectrum = analyze(fs, f, 1.0, size);
        let bin_hz = fs / size as f64;
        let found = (peak_bin(&spectrum) as f64 - size as f64 / 2.0) * bin_hz;
        assert!((found - f).abs() <= bin_hz, "found={found}");
    }

    #[test]
    fn spectrum_is_asymmetric_for_complex_tone() {
        let spectrum = analyze(1024.0, 100.0, 1.0, 1024);
        let pos = spectrum[512 + 100];
        let neg = spectrum[512 - 100];
        assert!(pos - neg > 100.0, "pos={pos} neg={neg}");
    }

    #[test]
    fn silence_hits_the_floor() {
        let mut analyzer = SpectrumAnalyzer::new(256).unwrap();
        let mut out = vec![0.0; 256];
        analyzer
            .process_db(&vec![Complex32::default(); 256], &mut out)
            .unwrap();
        assert!(out.iter().all(|v| (*v + 180.0).abs() < 0.01));
    }

    #[test]
    fn rejects_bad_sizes_and_lengths() {
        assert!(SpectrumAnalyzer::new(0).is_err());
        assert!(SpectrumAnalyzer::new(1000).is_err());
        let mut analyzer = SpectrumAnalyzer::new(64).unwrap();
        let mut out = vec![0.0; 64];
        assert!(analyzer
            .process_db(&vec![Complex32::default(); 32], &mut out)
            .is_err());
        assert!(analyzer
            .process_db(&vec![Complex32::default(); 64], &mut out[..10])
            .is_err());
    }

    #[test]
    fn repeated_processing_is_deterministic() {
        let mut osc = ComplexOscillator::new(1024.0, 80.0, 1.0).unwrap();
        let mut analyzer = SpectrumAnalyzer::new(1024).unwrap();
        let mut iq = vec![Complex32::default(); 1024];
        let mut a = vec![0.0; 1024];
        let mut b = vec![0.0; 1024];
        osc.fill(&mut iq);
        analyzer.process_db(&iq, &mut a).unwrap();
        analyzer.process_db(&iq, &mut b).unwrap();
        assert_eq!(a, b);
    }
}
