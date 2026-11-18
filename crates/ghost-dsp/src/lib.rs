#![allow(non_snake_case)]

use std::f32::consts::TAU;

pub fn generate_complex_tone(
    fs: f32,
    A: f32,
    freq: f32,
    mut phase: f32,
    out: &mut [[f32; 2]],
) -> f32 {
    let phase_step = TAU * freq / fs;

    for sample in out {
        let (q, i) = phase.sin_cos();
        *sample = [A * i, A * q];
        phase = (phase + phase_step).rem_euclid(TAU);
    }

    phase
}

pub fn mean_power(samples: &[[f32; 2]]) -> f32 {
    let mut sum = 0.0;

    for &[i, q] in samples {
        sum += i * i + q * q;
    }

    sum / samples.len() as f32
}

pub fn mean_power_db(samples: &[[f32; 2]]) -> f32 {
    power_to_db(mean_power(samples))
}

pub fn rms(samples: &[[f32; 2]]) -> f32 {
    mean_power(samples).sqrt()
}

pub fn power_to_db(power: f32) -> f32 {
    10.0 * power.log10()
}

#[cfg(test)]
mod complex_tone_tests {
    use super::*;

    #[test]
    fn generates_complex_tone() {
        let mut samples = [[0.0; 2]; 4];

        generate_complex_tone(4.0, 1.0, 1.0, 0.0, &mut samples);

        let expected = [[1.0, 0.0], [0.0, 1.0], [-1.0, 0.0], [0.0, -1.0]];

        for (actual, expected) in samples.iter().zip(expected) {
            for (a, e) in actual.iter().zip(expected) {
                assert!((a - e).abs() < 1e-6);
            }
        }
    }

    #[test]
    fn applies_amplitude() {
        let mut samples = [[0.0; 2]; 4];

        generate_complex_tone(4.0, 0.5, 1.0, 0.0, &mut samples);

        let expected = [[0.5, 0.0], [0.0, 0.5], [-0.5, 0.0], [0.0, -0.5]];

        for (actual, expected) in samples.iter().zip(expected) {
            for (a, e) in actual.iter().zip(expected) {
                assert!((a - e).abs() < 1e-6);
            }
        }
    }

    #[test]
    fn preserves_phase_continuity() {
        let mut full = [[0.0; 2]; 8];
        generate_complex_tone(8.0, 1.0, 1.0, 0.0, &mut full);

        let mut first = [[0.0; 2]; 3];
        let mut second = [[0.0; 2]; 5];

        let phase = generate_complex_tone(8.0, 1.0, 1.0, 0.0, &mut first);
        generate_complex_tone(8.0, 1.0, 1.0, phase, &mut second);

        for (actual, expected) in first.iter().chain(&second).zip(full) {
            for (a, e) in actual.iter().zip(expected) {
                assert!((a - e).abs() < 1e-6);
            }
        }
    }

    #[test]
    fn generates_negative_frequency() {
        let mut samples = [[0.0; 2]; 4];

        generate_complex_tone(4.0, 1.0, -1.0, 0.0, &mut samples);

        let expected = [[1.0, 0.0], [0.0, -1.0], [-1.0, 0.0], [0.0, 1.0]];

        for (actual, expected) in samples.iter().zip(expected) {
            for (a, e) in actual.iter().zip(expected) {
                assert!((a - e).abs() < 1e-6);
            }
        }
    }
}

#[cfg(test)]
mod power_tests {
    use super::*;

    #[test]
    fn calculates_mean_power() {
        let samples = [[3.0, 4.0], [0.0, 0.0]];

        let power = mean_power(&samples);

        assert_eq!(power, 12.5);
    }

    #[test]
    fn calculates_mean_power_db() {
        let samples = [[0.5, 0.0], [0.0, 0.5]];

        let power = mean_power_db(&samples);

        assert!((power + 6.0206).abs() < 1e-4);
    }

    #[test]
    fn calculates_rms() {
        let samples = [[3.0, 4.0], [0.0, 0.0]];

        let result = rms(&samples);

        assert!((result - 12.5_f32.sqrt()).abs() < 1e-6);
    }

    #[test]
    fn converts_power_to_db() {
        assert_eq!(power_to_db(1.0), 0.0);
        assert!((power_to_db(0.25) + 6.0206).abs() < 1e-4);
        assert!((power_to_db(4.0) - 6.0206).abs() < 1e-4);
    }
}
