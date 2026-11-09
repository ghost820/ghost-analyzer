use std::f64::consts::TAU;

use ghost_dsp::{Complex32, ComplexOscillator};
use ghost_engine::graph::{
    Block, BlockIo, CollectHandle, CollectSink, Gain, Graph, GraphError, PortType, Scheduler,
    SignalGenerator,
};

const SAMPLE_RATE: f64 = 48_000.0;
const FREQUENCY: f64 = 1_234.5;
const AMPLITUDE: f64 = 0.5;

struct Throttle {
    limit: usize,
}

impl Block for Throttle {
    fn input_ports(&self) -> &[PortType] {
        &[PortType::Complex32]
    }

    fn output_ports(&self) -> &[PortType] {
        &[PortType::Complex32]
    }

    fn process(&mut self, io: &mut BlockIo) {
        let (input, output) = io.input_output::<Complex32, Complex32>(0, 0);
        let count = input.len().min(output.len()).min(self.limit);
        output[..count].copy_from_slice(&input[..count]);
        if count > 0 {
            io.consume(0, count);
            io.produce(0, count);
        }
        if io.input_exhausted(0) {
            io.finish();
        }
    }
}

struct Stubborn;

impl Block for Stubborn {
    fn input_ports(&self) -> &[PortType] {
        &[PortType::Complex32]
    }

    fn process(&mut self, _io: &mut BlockIo) {}
}

fn oscillator() -> ComplexOscillator {
    ComplexOscillator::new(SAMPLE_RATE, FREQUENCY, AMPLITUDE).unwrap()
}

fn reference(count: usize, gain: f32) -> Vec<Complex32> {
    let mut samples = vec![Complex32::default(); count];
    oscillator().fill(&mut samples);
    samples.iter().map(|s| *s * gain).collect()
}

fn run_pipeline(
    total: u64,
    gain: f32,
    capacities: [usize; 2],
    throttle: Option<usize>,
) -> Vec<Complex32> {
    let mut graph = Graph::new();
    let generator = graph.add_block(SignalGenerator::new(oscillator(), Some(total)));
    let gain_block = graph.add_block(Gain::new(gain));
    let (sink, handle) = CollectSink::new(total as usize);
    let sink = graph.add_block(sink);
    match throttle {
        Some(limit) => {
            let throttle = graph.add_block(Throttle { limit });
            graph
                .connect(generator, 0, gain_block, 0, capacities[0])
                .unwrap();
            graph
                .connect(gain_block, 0, throttle, 0, capacities[1])
                .unwrap();
            graph.connect(throttle, 0, sink, 0, capacities[1]).unwrap();
        }
        None => {
            graph
                .connect(generator, 0, gain_block, 0, capacities[0])
                .unwrap();
            graph
                .connect(gain_block, 0, sink, 0, capacities[1])
                .unwrap();
        }
    }
    let mut scheduler = Scheduler::new(graph).unwrap();
    scheduler.run().unwrap();
    assert!(scheduler.is_complete());
    handle.samples()
}

fn assert_close(actual: &[Complex32], expected: &[Complex32]) {
    assert_eq!(actual.len(), expected.len());
    for (n, (a, e)) in actual.iter().zip(expected).enumerate() {
        assert!((a - e).norm() < 1e-6, "mismatch at {n}: {a} vs {e}");
    }
}

#[test]
fn generator_gain_sink_matches_reference() {
    let out = run_pipeline(10_000, 2.5, [1024, 1024], None);
    assert_close(&out, &reference(10_000, 2.5));
}

#[test]
fn sample_count_is_exact_for_unaligned_totals() {
    for total in [0u64, 1, 2, 7, 13, 1000, 1023, 1025] {
        let out = run_pipeline(total, 1.0, [16, 16], None);
        assert_eq!(out.len() as u64, total);
    }
}

#[test]
fn small_buffers_preserve_samples() {
    for capacities in [[1, 1], [2, 3], [7, 5], [3, 64], [64, 3]] {
        let out = run_pipeline(2_500, -1.5, capacities, None);
        assert_close(&out, &reference(2_500, -1.5));
    }
}

#[test]
fn partial_consumption_preserves_samples() {
    for limit in [1, 2, 3] {
        let out = run_pipeline(1_000, 0.75, [11, 9], Some(limit));
        assert_close(&out, &reference(1_000, 0.75));
    }
}

#[test]
fn phase_is_continuous_across_buffer_boundaries() {
    let out = run_pipeline(5_000, 1.0, [7, 5], None);
    let step = TAU * FREQUENCY / SAMPLE_RATE;
    for (n, pair) in out.windows(2).enumerate() {
        let delta = f64::from((pair[1] * pair[0].conj()).arg());
        let error = (delta - step).abs();
        assert!(error < 1e-3, "phase jump at {n}: {delta} vs {step}");
        assert!((f64::from(pair[1].norm()) - AMPLITUDE).abs() < 1e-5);
    }
}

#[test]
fn gain_applies_to_every_sample() {
    let base = run_pipeline(512, 1.0, [8, 8], None);
    let scaled = run_pipeline(512, 4.0, [8, 8], None);
    for (b, s) in base.iter().zip(&scaled) {
        assert!((*b * 4.0 - s).norm() < 1e-6);
    }
}

#[test]
fn step_reports_running_then_complete() {
    let mut graph = Graph::new();
    let generator = graph.add_block(SignalGenerator::new(oscillator(), Some(100)));
    let (sink, handle) = CollectSink::new(100);
    let sink = graph.add_block(sink);
    graph.connect(generator, 0, sink, 0, 10).unwrap();
    let mut scheduler = Scheduler::new(graph).unwrap();
    let mut steps = 0;
    while scheduler.step().unwrap() == ghost_engine::graph::Status::Running {
        steps += 1;
        assert!(steps < 1000);
    }
    assert_eq!(handle.len(), 100);
    assert_eq!(
        scheduler.step().unwrap(),
        ghost_engine::graph::Status::Complete
    );
}

#[test]
fn infinite_generator_with_stalled_consumer_reports_stall() {
    let mut graph = Graph::new();
    let generator = graph.add_block(SignalGenerator::new(oscillator(), None));
    let stubborn = graph.add_block(Stubborn);
    graph.connect(generator, 0, stubborn, 0, 8).unwrap();
    let mut scheduler = Scheduler::new(graph).unwrap();
    assert_eq!(scheduler.run(), Err(GraphError::Stalled));
}

#[test]
fn unconnected_ports_are_rejected() {
    let mut graph = Graph::new();
    graph.add_block(SignalGenerator::new(oscillator(), Some(10)));
    assert!(matches!(
        Scheduler::new(graph),
        Err(GraphError::UnconnectedPort { .. })
    ));
}

#[test]
fn connection_validation() {
    let mut graph = Graph::new();
    let generator = graph.add_block(SignalGenerator::new(oscillator(), Some(10)));
    let gain_a = graph.add_block(Gain::new(1.0));
    let gain_b = graph.add_block(Gain::new(1.0));
    let (sink, _handle): (CollectSink, CollectHandle) = CollectSink::new(0);
    let sink = graph.add_block(sink);

    assert_eq!(
        graph.connect(generator, 0, gain_a, 0, 0),
        Err(GraphError::ZeroCapacity)
    );
    assert!(matches!(
        graph.connect(generator, 1, gain_a, 0, 4),
        Err(GraphError::InvalidPort { .. })
    ));
    assert!(matches!(
        graph.connect(sink, 0, gain_a, 0, 4),
        Err(GraphError::InvalidPort { .. })
    ));
    graph.connect(generator, 0, gain_a, 0, 4).unwrap();
    assert!(matches!(
        graph.connect(generator, 0, gain_b, 0, 4),
        Err(GraphError::PortAlreadyConnected { .. })
    ));
    graph.connect(gain_a, 0, gain_b, 0, 4).unwrap();
    assert_eq!(
        graph.connect(gain_b, 0, gain_a, 0, 4),
        Err(GraphError::PortAlreadyConnected {
            block: gain_a,
            direction: ghost_engine::graph::PortDirection::Input,
            port: 0
        })
    );
    assert_eq!(
        graph.connect(gain_b, 0, gain_b, 0, 4),
        Err(GraphError::PortAlreadyConnected {
            block: gain_b,
            direction: ghost_engine::graph::PortDirection::Input,
            port: 0
        })
    );
}

#[test]
fn cycles_are_rejected() {
    let mut graph = Graph::new();
    let a = graph.add_block(Gain::new(1.0));
    let b = graph.add_block(Gain::new(1.0));
    graph.connect(a, 0, b, 0, 4).unwrap();
    assert_eq!(graph.connect(b, 0, a, 0, 4), Err(GraphError::Cycle));
    let c = graph.add_block(Gain::new(1.0));
    assert_eq!(graph.connect(c, 0, c, 0, 4), Err(GraphError::Cycle));
}
