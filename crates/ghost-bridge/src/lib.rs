use ghost_engine::{Engine, EngineConfig};
use napi::bindgen_prelude::Float32Array;
use napi::{Error, Result, Status};
use napi_derive::napi;

#[napi(object)]
pub struct EngineOptions {
    pub sample_rate: f64,
    pub frequency: f64,
    pub amplitude: f64,
    pub block_size: u32,
}

#[napi(object)]
pub struct BlockInfo {
    pub sequence: f64,
    pub process_micros: f64,
}

#[napi]
pub struct GhostEngine {
    inner: Engine,
}

fn to_napi(err: ghost_engine::DspError) -> Error {
    Error::new(Status::InvalidArg, err.to_string())
}

#[napi]
impl GhostEngine {
    #[napi(constructor)]
    pub fn new(options: EngineOptions) -> Result<Self> {
        let inner = Engine::new(EngineConfig {
            sample_rate: options.sample_rate,
            frequency: options.frequency,
            amplitude: options.amplitude,
            block_size: options.block_size as usize,
        })
        .map_err(to_napi)?;
        Ok(Self { inner })
    }

    #[napi(getter)]
    pub fn block_size(&self) -> u32 {
        self.inner.block_size() as u32
    }

    #[napi]
    pub fn set_sample_rate(&mut self, hz: f64) -> Result<()> {
        self.inner.set_sample_rate(hz).map_err(to_napi)
    }

    #[napi]
    pub fn set_frequency(&mut self, hz: f64) -> Result<()> {
        self.inner.set_frequency(hz).map_err(to_napi)
    }

    #[napi]
    pub fn set_amplitude(&mut self, amplitude: f64) -> Result<()> {
        self.inner.set_amplitude(amplitude).map_err(to_napi)
    }

    #[napi]
    pub fn process_into(
        &mut self,
        mut iq: Float32Array,
        mut spectrum_db: Float32Array,
    ) -> Result<BlockInfo> {
        let n = self.inner.block_size();
        if iq.len() != n * 2 || spectrum_db.len() != n {
            return Err(Error::new(
                Status::InvalidArg,
                format!(
                    "expected iq length {} and spectrum length {n}, got {} and {}",
                    n * 2,
                    iq.len(),
                    spectrum_db.len()
                ),
            ));
        }
        let started = std::time::Instant::now();
        self.inner.process_block().map_err(to_napi)?;
        let elapsed = started.elapsed();
        unsafe { iq.as_mut() }.copy_from_slice(self.inner.iq_interleaved());
        unsafe { spectrum_db.as_mut() }.copy_from_slice(self.inner.spectrum_db());
        Ok(BlockInfo {
            sequence: self.inner.blocks_processed() as f64,
            process_micros: elapsed.as_secs_f64() * 1e6,
        })
    }
}
