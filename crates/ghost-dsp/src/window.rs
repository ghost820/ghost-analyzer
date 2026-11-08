use std::f64::consts::TAU;

pub fn hann_periodic(size: usize) -> Vec<f32> {
    (0..size)
        .map(|n| (0.5 - 0.5 * (TAU * n as f64 / size as f64).cos()) as f32)
        .collect()
}
