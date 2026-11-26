use triple_buffer::{Input, Output};

pub const PLOT_SAMPLES: usize = 1024;

pub type PlotReader = Output<[f32; PLOT_SAMPLES]>;

pub struct PlotTap {
    producer: Input<[f32; PLOT_SAMPLES]>,
    position: usize,
    channels: usize,
}

impl PlotTap {
    pub fn new(channels: u16) -> (Self, PlotReader) {
        let (producer, consumer) = triple_buffer::triple_buffer(&[0.0; PLOT_SAMPLES]);

        (
            Self {
                producer,
                position: 0,
                channels: usize::from(channels),
            },
            consumer,
        )
    }

    pub fn process(&mut self, input: &[f32]) {
        for frame in input.chunks_exact(self.channels) {
            self.producer.input_buffer_mut()[self.position] = frame[0];

            self.position += 1;

            if self.position == PLOT_SAMPLES {
                self.producer.publish();
                self.position = 0;
            }
        }
    }
}
