use hound::{SampleFormat, WavReader};

pub struct FileSource {
    pub sample_rate: u32,
    pub channels: u16,
    samples: Vec<f32>,
    position: usize,
}

impl FileSource {
    pub fn new(path: &str) -> Self {
        let mut reader = WavReader::open(path).unwrap();
        let spec = reader.spec();

        let samples: Vec<f32> = match spec.sample_format {
            SampleFormat::Float => reader.samples::<f32>().map(|s| s.unwrap()).collect(),
            SampleFormat::Int => {
                let scale = 2.0_f32.powi(i32::from(spec.bits_per_sample) - 1);

                reader
                    .samples::<i32>()
                    .map(|s| s.unwrap() as f32 / scale)
                    .collect()
            }
        };

        Self {
            sample_rate: spec.sample_rate,
            channels: spec.channels,
            samples,
            position: 0,
        }
    }

    pub fn process(&mut self, mut output: &mut [f32]) {
        while !output.is_empty() {
            let count = output.len().min(self.samples.len() - self.position);
            let (head, tail) = output.split_at_mut(count);

            head.copy_from_slice(&self.samples[self.position..self.position + count]);

            self.position += count;

            if self.position == self.samples.len() {
                self.position = 0;
            }

            output = tail;
        }
    }
}
