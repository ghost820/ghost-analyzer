use ghost_dsp::Complex32;

use super::ring::RingBuffer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortType {
    Complex32,
}

pub enum Buffer {
    Complex32(RingBuffer<Complex32>),
}

impl Buffer {
    pub fn new(port_type: PortType, capacity: usize) -> Self {
        match port_type {
            PortType::Complex32 => Self::Complex32(RingBuffer::new(capacity)),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Self::Complex32(ring) => ring.len(),
        }
    }

    pub fn consume(&mut self, count: usize) {
        match self {
            Self::Complex32(ring) => ring.consume(count),
        }
    }

    pub fn produce(&mut self, count: usize) {
        match self {
            Self::Complex32(ring) => ring.produce(count),
        }
    }
}

pub trait Sample: Copy + Default + 'static {
    const PORT_TYPE: PortType;
    fn ring(buffer: &Buffer) -> Option<&RingBuffer<Self>>;
    fn ring_mut(buffer: &mut Buffer) -> Option<&mut RingBuffer<Self>>;
}

impl Sample for Complex32 {
    const PORT_TYPE: PortType = PortType::Complex32;

    fn ring(buffer: &Buffer) -> Option<&RingBuffer<Self>> {
        match buffer {
            Buffer::Complex32(ring) => Some(ring),
        }
    }

    fn ring_mut(buffer: &mut Buffer) -> Option<&mut RingBuffer<Self>> {
        match buffer {
            Buffer::Complex32(ring) => Some(ring),
        }
    }
}

pub struct Edge {
    pub buffer: Buffer,
    pub closed: bool,
    pub from: usize,
    pub to: usize,
}

pub trait Block {
    fn input_ports(&self) -> &[PortType] {
        &[]
    }

    fn output_ports(&self) -> &[PortType] {
        &[]
    }

    fn process(&mut self, io: &mut BlockIo);
}

pub struct BlockIo<'a> {
    edges: &'a mut [Edge],
    inputs: &'a [usize],
    outputs: &'a [usize],
    progressed: bool,
    finished: bool,
}

const TYPE_MISMATCH: &str = "block accessed a port with the wrong sample type";

impl<'a> BlockIo<'a> {
    pub(crate) fn new(edges: &'a mut [Edge], inputs: &'a [usize], outputs: &'a [usize]) -> Self {
        Self {
            edges,
            inputs,
            outputs,
            progressed: false,
            finished: false,
        }
    }

    pub(crate) fn progressed(&self) -> bool {
        self.progressed
    }

    pub(crate) fn finished(&self) -> bool {
        self.finished
    }

    pub fn input<T: Sample>(&self, port: usize) -> &[T] {
        let edge = &self.edges[self.inputs[port]];
        T::ring(&edge.buffer).expect(TYPE_MISMATCH).readable()
    }

    pub fn output<T: Sample>(&mut self, port: usize) -> &mut [T] {
        let edge = &mut self.edges[self.outputs[port]];
        T::ring_mut(&mut edge.buffer)
            .expect(TYPE_MISMATCH)
            .writable()
    }

    pub fn input_output<I: Sample, O: Sample>(
        &mut self,
        input_port: usize,
        output_port: usize,
    ) -> (&[I], &mut [O]) {
        let input_index = self.inputs[input_port];
        let output_index = self.outputs[output_port];
        assert_ne!(input_index, output_index, "input and output share an edge");
        let (input_edge, output_edge) = if input_index < output_index {
            let (low, high) = self.edges.split_at_mut(output_index);
            (&low[input_index], &mut high[0])
        } else {
            let (low, high) = self.edges.split_at_mut(input_index);
            (&high[0], &mut low[output_index])
        };
        let input = I::ring(&input_edge.buffer).expect(TYPE_MISMATCH).readable();
        let output = O::ring_mut(&mut output_edge.buffer)
            .expect(TYPE_MISMATCH)
            .writable();
        (input, output)
    }

    pub fn input_available(&self, port: usize) -> usize {
        self.edges[self.inputs[port]].buffer.len()
    }

    pub fn input_exhausted(&self, port: usize) -> bool {
        let edge = &self.edges[self.inputs[port]];
        edge.closed && edge.buffer.len() == 0
    }

    pub fn consume(&mut self, port: usize, count: usize) {
        self.edges[self.inputs[port]].buffer.consume(count);
        self.progressed |= count > 0;
    }

    pub fn produce(&mut self, port: usize, count: usize) {
        self.edges[self.outputs[port]].buffer.produce(count);
        self.progressed |= count > 0;
    }

    pub fn finish(&mut self) {
        for &index in self.outputs {
            self.edges[index].closed = true;
        }
        self.finished = true;
        self.progressed = true;
    }
}
