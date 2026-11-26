mod dsp;
mod graph_compiler;

use std::time::Duration;

use eframe::egui::{Button, CentralPanel, Color32, DragValue, Frame, Key, Pos2, Ui, Visuals};
use eframe::{App, NativeOptions};
use egui_plot::{Line, Plot, PlotPoints};
use egui_snarl::ui::{PinInfo, SnarlPin, SnarlStyle, SnarlViewer, SnarlWidget};
use egui_snarl::{InPin, NodeId, OutPin, Snarl};

use dsp::{PLOT_SAMPLES, PlotReader};
use graph_compiler::CompiledGraph;

enum Node {
    FileSource { path: String },
    Gain(dsp::Gain),
    AudioSink,
    TimePlot(Option<PlotData>),
    FftPlot(Option<FftPlotData>),
}

struct PlotData {
    reader: PlotReader,
    sample_rate: u32,
}

struct FftPlotData {
    plot: PlotData,
    spectrum: dsp::Spectrum,
}

struct GraphViewer;

impl SnarlViewer<Node> for GraphViewer {
    fn title(&mut self, node: &Node) -> String {
        match node {
            Node::FileSource { .. } => "File Source".into(),
            Node::Gain(_) => "Gain".into(),
            Node::AudioSink => "Audio Sink".into(),
            Node::TimePlot(_) => "Time Plot".into(),
            Node::FftPlot(_) => "FFT Plot".into(),
        }
    }

    fn inputs(&mut self, node: &Node) -> usize {
        match node {
            Node::FileSource { .. } => 0,
            Node::Gain(_) | Node::AudioSink | Node::TimePlot(_) | Node::FftPlot(_) => 1,
        }
    }

    fn outputs(&mut self, node: &Node) -> usize {
        match node {
            Node::FileSource { .. } | Node::Gain(_) => 1,
            Node::AudioSink | Node::TimePlot(_) | Node::FftPlot(_) => 0,
        }
    }

    fn show_input(
        &mut self,
        _pin: &InPin,
        ui: &mut Ui,
        _snarl: &mut Snarl<Node>,
    ) -> impl SnarlPin + 'static {
        ui.label("In");
        PinInfo::default()
    }

    fn show_output(
        &mut self,
        _pin: &OutPin,
        ui: &mut Ui,
        _snarl: &mut Snarl<Node>,
    ) -> impl SnarlPin + 'static {
        ui.label("Out");
        PinInfo::default()
    }

    fn has_body(&mut self, node: &Node) -> bool {
        !matches!(node, Node::AudioSink)
    }

    fn show_body(
        &mut self,
        node: NodeId,
        _inputs: &[InPin],
        _outputs: &[OutPin],
        ui: &mut Ui,
        snarl: &mut Snarl<Node>,
    ) {
        match &mut snarl[node] {
            Node::FileSource { path } => {
                ui.label("Path:");
                ui.text_edit_singleline(path);
            }
            Node::Gain(dsp) => {
                let mut value = dsp.gain();

                if ui
                    .add(DragValue::new(&mut value).speed(0.01).prefix("Gain: "))
                    .changed()
                {
                    dsp.set_gain(value);
                }
            }
            Node::AudioSink => {}
            Node::TimePlot(data) => {
                let sample_rate = data
                    .as_ref()
                    .map_or(48000.0, |data| f64::from(data.sample_rate));

                let points = match data {
                    Some(data) => {
                        ui.ctx().request_repaint_after(Duration::from_millis(16));

                        data.reader
                            .read()
                            .iter()
                            .enumerate()
                            .map(|(i, &sample)| [i as f64 / sample_rate, f64::from(sample)])
                            .collect()
                    }
                    None => PlotPoints::default(),
                };

                Plot::new(("time_plot", node))
                    .width(300.0)
                    .height(150.0)
                    .x_axis_label("Time [s]")
                    .show(ui, |plot_ui| {
                        plot_ui.set_plot_bounds_x(0.0..=PLOT_SAMPLES as f64 / sample_rate);
                        plot_ui.set_plot_bounds_y(-1.0..=1.0);
                        plot_ui.line(Line::new("Signal", points));
                    });
            }
            Node::FftPlot(data) => {
                let sample_rate = data
                    .as_ref()
                    .map_or(48000.0, |data| f64::from(data.plot.sample_rate));

                let frequency_step = sample_rate / PLOT_SAMPLES as f64;

                let points = match data {
                    Some(data) => {
                        ui.ctx().request_repaint_after(Duration::from_millis(16));

                        data.spectrum
                            .process(data.plot.reader.read())
                            .enumerate()
                            .map(|(i, amplitude)| [i as f64 * frequency_step, f64::from(amplitude)])
                            .collect()
                    }
                    None => PlotPoints::default(),
                };

                Plot::new(("fft_plot", node))
                    .width(300.0)
                    .height(150.0)
                    .x_axis_label("Frequency [Hz]")
                    .show(ui, |plot_ui| {
                        plot_ui.set_plot_bounds_x(0.0..=sample_rate / 2.0);
                        plot_ui.set_plot_bounds_y(0.0..=1.0);
                        plot_ui.line(Line::new("Spectrum", points));
                    });
            }
        }
    }

    fn has_graph_menu(&mut self, _pos: Pos2, _snarl: &mut Snarl<Node>) -> bool {
        true
    }

    fn show_graph_menu(&mut self, pos: Pos2, ui: &mut Ui, snarl: &mut Snarl<Node>) {
        if ui.button("File Source").clicked() {
            snarl.insert_node(
                pos,
                Node::FileSource {
                    path: String::new(),
                },
            );
            ui.close();
        }

        if ui.button("Gain").clicked() {
            snarl.insert_node(pos, Node::Gain(dsp::Gain::new(1.0)));
            ui.close();
        }

        if ui.button("Audio Sink").clicked() {
            snarl.insert_node(pos, Node::AudioSink);
            ui.close();
        }

        if ui.button("Time Plot").clicked() {
            snarl.insert_node(pos, Node::TimePlot(None));
            ui.close();
        }

        if ui.button("FFT Plot").clicked() {
            snarl.insert_node(pos, Node::FftPlot(None));
            ui.close();
        }
    }

    fn connect(&mut self, from: &OutPin, to: &InPin, snarl: &mut Snarl<Node>) {
        snarl.drop_inputs(to.id);
        snarl.connect(from.id, to.id);
    }
}

struct GhostAnalyzer {
    graph: Snarl<Node>,
    audio_sink: Option<dsp::AudioSink>,
}

impl GhostAnalyzer {
    fn start(&mut self) {
        let CompiledGraph {
            mut runtime,
            sample_rate,
            channels,
            plot_readers,
        } = graph_compiler::compile(&self.graph);

        for (id, reader) in plot_readers {
            let data = PlotData {
                reader,
                sample_rate,
            };

            match &mut self.graph[id] {
                Node::TimePlot(plot_data) => {
                    *plot_data = Some(data);
                }
                Node::FftPlot(plot_data) => {
                    *plot_data = Some(FftPlotData {
                        plot: data,
                        spectrum: dsp::Spectrum::new(PLOT_SAMPLES),
                    });
                }
                _ => unreachable!(),
            }
        }

        self.audio_sink = Some(dsp::AudioSink::start(
            sample_rate,
            channels,
            move |output| runtime.process(output),
        ));
    }

    fn stop(&mut self) {
        self.audio_sink = None;

        for node in self.graph.nodes_mut() {
            match node {
                Node::TimePlot(data) => *data = None,
                Node::FftPlot(data) => *data = None,
                _ => {}
            }
        }
    }
}

impl App for GhostAnalyzer {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        CentralPanel::default()
            .frame(Frame::central_panel(ui.style()).fill(Color32::from_rgb(9, 12, 18)))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let running = self.audio_sink.is_some();

                    if ui.add_enabled(!running, Button::new("Start")).clicked() {
                        self.start();
                    }

                    if ui.add_enabled(running, Button::new("Stop")).clicked() {
                        self.stop();
                    }
                });

                ui.separator();

                let mut style = SnarlStyle::new();
                style.bg_frame = Some(Frame::NONE);

                let widget = SnarlWidget::new().id_salt("graph").style(style);

                widget.show(&mut self.graph, &mut GraphViewer, ui);

                if !ui.ctx().egui_wants_keyboard_input()
                    && ui.input(|input| input.key_pressed(Key::Delete))
                {
                    for node in widget.get_selected_nodes(ui) {
                        self.graph.remove_node(node);
                    }
                }
            });
    }
}

fn main() -> eframe::Result {
    eframe::run_native(
        "GhostAnalyzer",
        NativeOptions::default(),
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(Visuals::dark());
            Ok(Box::new(GhostAnalyzer {
                graph: Snarl::new(),
                audio_sink: None,
            }))
        }),
    )
}
