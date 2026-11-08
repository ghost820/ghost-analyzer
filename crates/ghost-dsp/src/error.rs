use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum DspError {
    InvalidSampleRate(f64),
    InvalidFrequency(f64),
    InvalidAmplitude(f64),
    InvalidFftSize(usize),
    LengthMismatch { expected: usize, actual: usize },
}

impl fmt::Display for DspError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSampleRate(v) => {
                write!(f, "sample rate must be finite and positive, got {v}")
            }
            Self::InvalidFrequency(v) => write!(f, "frequency must be finite, got {v}"),
            Self::InvalidAmplitude(v) => {
                write!(f, "amplitude must be finite and non-negative, got {v}")
            }
            Self::InvalidFftSize(v) => {
                write!(f, "FFT size must be a power of two of at least 2, got {v}")
            }
            Self::LengthMismatch { expected, actual } => {
                write!(
                    f,
                    "buffer length mismatch: expected {expected}, got {actual}"
                )
            }
        }
    }
}

impl std::error::Error for DspError {}
