use std::f64::consts::TAU;

use num_complex::Complex32;

use crate::DspError;

#[derive(Debug, Clone)]
pub struct ComplexOscillator {
    sample_rate: f64,
    frequency: f64,
    amplitude: f32,
    phase: f64,
    phase_step: f64,
}

impl ComplexOscillator {
    pub fn new(sample_rate: f64, frequency: f64, amplitude: f64) -> Result<Self, DspError> {
        validate_sample_rate(sample_rate)?;
        validate_frequency(frequency)?;
        validate_amplitude(amplitude)?;
        Ok(Self {
            sample_rate,
            frequency,
            amplitude: amplitude as f32,
            phase: 0.0,
            phase_step: TAU * frequency / sample_rate,
        })
    }

    pub fn sample_rate(&self) -> f64 {
        self.sample_rate
    }

    pub fn frequency(&self) -> f64 {
        self.frequency
    }

    pub fn amplitude(&self) -> f64 {
        f64::from(self.amplitude)
    }

    pub fn phase(&self) -> f64 {
        self.phase
    }

    pub fn set_sample_rate(&mut self, sample_rate: f64) -> Result<(), DspError> {
        validate_sample_rate(sample_rate)?;
        self.sample_rate = sample_rate;
        self.update_step();
        Ok(())
    }

    pub fn set_frequency(&mut self, frequency: f64) -> Result<(), DspError> {
        validate_frequency(frequency)?;
        self.frequency = frequency;
        self.update_step();
        Ok(())
    }

    pub fn set_amplitude(&mut self, amplitude: f64) -> Result<(), DspError> {
        validate_amplitude(amplitude)?;
        self.amplitude = amplitude as f32;
        Ok(())
    }

    pub fn reset_phase(&mut self) {
        self.phase = 0.0;
    }

    pub fn fill(&mut self, out: &mut [Complex32]) {
        let amplitude = self.amplitude;
        let step = self.phase_step;
        let mut phase = self.phase;
        for sample in out.iter_mut() {
            let (sin, cos) = (phase as f32).sin_cos();
            *sample = Complex32::new(amplitude * cos, amplitude * sin);
            phase += step;
            if phase >= TAU {
                phase -= TAU;
            }
        }
        self.phase = phase.rem_euclid(TAU);
    }

    fn update_step(&mut self) {
        self.phase_step = (TAU * self.frequency / self.sample_rate).rem_euclid(TAU);
    }
}

fn validate_sample_rate(v: f64) -> Result<(), DspError> {
    if v.is_finite() && v > 0.0 {
        Ok(())
    } else {
        Err(DspError::InvalidSampleRate(v))
    }
}

fn validate_frequency(v: f64) -> Result<(), DspError> {
    if v.is_finite() {
        Ok(())
    } else {
        Err(DspError::InvalidFrequency(v))
    }
}

fn validate_amplitude(v: f64) -> Result<(), DspError> {
    if v.is_finite() && v >= 0.0 {
        Ok(())
    } else {
        Err(DspError::InvalidAmplitude(v))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f32 = 1e-4;

    fn expected(n: usize, fs: f64, f: f64, a: f64) -> Complex32 {
        let phase = TAU * f * n as f64 / fs;
        Complex32::new((a * phase.cos()) as f32, (a * phase.sin()) as f32)
    }

    #[test]
    fn generates_expected_samples() {
        let (fs, f, a) = (48_000.0, 1_000.0, 0.75);
        let mut osc = ComplexOscillator::new(fs, f, a).unwrap();
        let mut buf = vec![Complex32::default(); 256];
        osc.fill(&mut buf);
        for (n, s) in buf.iter().enumerate() {
            let e = expected(n, fs, f, a);
            assert!(
                (s.re - e.re).abs() < EPS && (s.im - e.im).abs() < EPS,
                "n={n}"
            );
        }
    }

    #[test]
    fn magnitude_equals_amplitude() {
        let mut osc = ComplexOscillator::new(1_000_000.0, 123_456.0, 0.4).unwrap();
        let mut buf = vec![Complex32::default(); 4096];
        osc.fill(&mut buf);
        assert!(buf.iter().all(|s| (s.norm() - 0.4).abs() < EPS));
    }

    #[test]
    fn phase_is_continuous_across_buffers() {
        let (fs, f, a) = (44_100.0, 3_333.3, 1.0);
        let mut whole = ComplexOscillator::new(fs, f, a).unwrap();
        let mut chunked = ComplexOscillator::new(fs, f, a).unwrap();

        let mut reference = vec![Complex32::default(); 3000];
        whole.fill(&mut reference);

        let mut joined = Vec::new();
        for size in [1, 7, 512, 1000, 1480] {
            let mut part = vec![Complex32::default(); size];
            chunked.fill(&mut part);
            joined.extend(part);
        }
        assert_eq!(joined.len(), reference.len());
        for (n, (a, b)) in joined.iter().zip(&reference).enumerate() {
            assert!((a - b).norm() < EPS, "discontinuity at {n}");
        }
    }

    #[test]
    fn phase_survives_parameter_changes() {
        let mut osc = ComplexOscillator::new(48_000.0, 1_000.0, 1.0).unwrap();
        let mut buf = vec![Complex32::default(); 100];
        osc.fill(&mut buf);
        let last = *buf.last().unwrap();
        let phase_before = osc.phase();

        osc.set_frequency(5_000.0).unwrap();
        osc.set_sample_rate(96_000.0).unwrap();
        osc.set_amplitude(0.5).unwrap();
        assert_eq!(osc.phase(), phase_before);

        let mut next = vec![Complex32::default(); 1];
        osc.fill(&mut next);
        let expected_phase = phase_before;
        assert!((next[0].re - 0.5 * expected_phase.cos() as f32).abs() < EPS);
        assert!((next[0].im - 0.5 * expected_phase.sin() as f32).abs() < EPS);
        let step = TAU * 1_000.0 / 48_000.0;
        let prev_phase = (expected_phase - step).rem_euclid(TAU);
        assert!((last.re - prev_phase.cos() as f32).abs() < EPS);
    }

    #[test]
    fn negative_frequency_rotates_clockwise() {
        let mut osc = ComplexOscillator::new(8.0, -1.0, 1.0).unwrap();
        let mut buf = vec![Complex32::default(); 3];
        osc.fill(&mut buf);
        assert!(buf[1].im < 0.0);
        assert!((buf[2].im + 1.0).abs() < EPS);
    }

    #[test]
    fn phase_stays_bounded_over_long_runs() {
        let mut osc = ComplexOscillator::new(1_000_000.0, 333_333.0, 1.0).unwrap();
        let mut buf = vec![Complex32::default(); 4096];
        for _ in 0..500 {
            osc.fill(&mut buf);
            assert!((0.0..TAU).contains(&osc.phase()));
        }
    }

    #[test]
    fn rejects_invalid_parameters() {
        assert!(ComplexOscillator::new(0.0, 1.0, 1.0).is_err());
        assert!(ComplexOscillator::new(f64::NAN, 1.0, 1.0).is_err());
        assert!(ComplexOscillator::new(1.0, f64::INFINITY, 1.0).is_err());
        assert!(ComplexOscillator::new(1.0, 0.1, -1.0).is_err());
        let mut osc = ComplexOscillator::new(1.0, 0.1, 1.0).unwrap();
        assert!(osc.set_sample_rate(-5.0).is_err());
        assert_eq!(osc.sample_rate(), 1.0);
    }
}
