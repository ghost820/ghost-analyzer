use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, Stream, StreamConfig};

pub struct AudioSink {
    _stream: Stream,
}

impl AudioSink {
    pub fn start(
        sample_rate: u32,
        channels: u16,
        mut process: impl FnMut(&mut [f32]) + Send + 'static,
    ) -> Self {
        let host = cpal::default_host();
        let device = host.default_output_device().unwrap();

        let config = StreamConfig {
            channels,
            sample_rate,
            buffer_size: BufferSize::Default,
        };

        // TODO: log() instead of println.
        let stream = device
            .build_output_stream(
                config,
                move |output: &mut [f32], _| process(output),
                |err| eprintln!("{err}"),
                None,
            )
            .unwrap();

        stream.play().unwrap();

        Self { _stream: stream }
    }
}
