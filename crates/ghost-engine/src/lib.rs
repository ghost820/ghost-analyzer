pub mod graph;

use ghost_dsp::{Complex32, ComplexOscillator, SpectrumAnalyzer};

pub use ghost_dsp::DspError;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EngineConfig {
    pub sample_rate: f64,
    pub frequency: f64,
    pub amplitude: f64,
    pub block_size: usize,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            sample_rate: 48_000.0,
            frequency: 3_000.0,
            amplitude: 0.8,
            block_size: 2048,
        }
    }
}

pub struct Engine {
    oscillator: ComplexOscillator,
    analyzer: SpectrumAnalyzer,
    iq: Vec<Complex32>,
    spectrum_db: Vec<f32>,
    blocks_processed: u64,
}

impl Engine {
    pub fn new(config: EngineConfig) -> Result<Self, DspError> {
        let analyzer = SpectrumAnalyzer::new(config.block_size)?;
        let oscillator =
            ComplexOscillator::new(config.sample_rate, config.frequency, config.amplitude)?;
        Ok(Self {
            oscillator,
            analyzer,
            iq: vec![Complex32::default(); config.block_size],
            spectrum_db: vec![0.0; config.block_size],
            blocks_processed: 0,
        })
    }

    pub fn block_size(&self) -> usize {
        self.iq.len()
    }

    pub fn sample_rate(&self) -> f64 {
        self.oscillator.sample_rate()
    }

    pub fn frequency(&self) -> f64 {
        self.oscillator.frequency()
    }

    pub fn amplitude(&self) -> f64 {
        self.oscillator.amplitude()
    }

    pub fn blocks_processed(&self) -> u64 {
        self.blocks_processed
    }

    pub fn set_sample_rate(&mut self, sample_rate: f64) -> Result<(), DspError> {
        self.oscillator.set_sample_rate(sample_rate)
    }

    pub fn set_frequency(&mut self, frequency: f64) -> Result<(), DspError> {
        self.oscillator.set_frequency(frequency)
    }

    pub fn set_amplitude(&mut self, amplitude: f64) -> Result<(), DspError> {
        self.oscillator.set_amplitude(amplitude)
    }

    pub fn process_block(&mut self) -> Result<(), DspError> {
        self.oscillator.fill(&mut self.iq);
        self.analyzer.process_db(&self.iq, &mut self.spectrum_db)?;
        self.blocks_processed += 1;
        Ok(())
    }

    pub fn iq(&self) -> &[Complex32] {
        &self.iq
    }

    pub fn iq_interleaved(&self) -> &[f32] {
        bytemuck::cast_slice(&self.iq)
    }

    pub fn spectrum_db(&self) -> &[f32] {
        &self.spectrum_db
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn processes_continuous_blocks() {
        let config = EngineConfig {
            sample_rate: 8192.0,
            frequency: 1000.0,
            amplitude: 1.0,
            block_size: 1024,
        };
        let mut engine = Engine::new(config).unwrap();
        let mut reference = ComplexOscillator::new(8192.0, 1000.0, 1.0).unwrap();

        for _ in 0..5 {
            engine.process_block().unwrap();
            let mut expected = vec![Complex32::default(); 1024];
            reference.fill(&mut expected);
            assert_eq!(engine.iq(), expected.as_slice());
        }
        assert_eq!(engine.blocks_processed(), 5);
    }

    #[test]
    fn interleaved_view_matches_complex_samples() {
        let mut engine = Engine::new(EngineConfig::default()).unwrap();
        engine.process_block().unwrap();
        let flat = engine.iq_interleaved();
        assert_eq!(flat.len(), engine.iq().len() * 2);
        for (i, s) in engine.iq().iter().enumerate() {
            assert_eq!(flat[2 * i], s.re);
            assert_eq!(flat[2 * i + 1], s.im);
        }
    }

    #[test]
    fn spectrum_tracks_parameter_changes() {
        let mut engine = Engine::new(EngineConfig {
            sample_rate: 4096.0,
            frequency: 512.0,
            amplitude: 1.0,
            block_size: 4096,
        })
        .unwrap();
        let peak = |engine: &Engine| {
            engine
                .spectrum_db()
                .iter()
                .enumerate()
                .max_by(|a, b| a.1.total_cmp(b.1))
                .map(|(i, v)| (i, *v))
                .unwrap()
        };

        engine.process_block().unwrap();
        assert_eq!(peak(&engine).0, 2048 + 512);

        engine.set_frequency(-256.0).unwrap();
        engine.process_block().unwrap();
        assert_eq!(peak(&engine).0, 2048 - 256);

        engine.set_amplitude(0.1).unwrap();
        engine.process_block().unwrap();
        assert!((peak(&engine).1 + 20.0).abs() < 0.1);
    }

    #[test]
    fn invalid_configuration_is_rejected() {
        let bad = EngineConfig {
            block_size: 1000,
            ..EngineConfig::default()
        };
        assert!(Engine::new(bad).is_err());
        let mut engine = Engine::new(EngineConfig::default()).unwrap();
        assert!(engine.set_sample_rate(0.0).is_err());
        assert!(engine.set_amplitude(-1.0).is_err());
        assert_eq!(engine.sample_rate(), 48_000.0);
    }
}
