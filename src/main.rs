use eframe::egui::{CentralPanel, Color32, DragValue, Frame, Key, Pos2, Ui, Visuals};
use eframe::{App, NativeOptions};
use egui_snarl::ui::{PinInfo, SnarlPin, SnarlStyle, SnarlViewer, SnarlWidget};
use egui_snarl::{InPin, NodeId, OutPin, Snarl};

enum Node {
    FileSource { path: String },
    Gain(f32),
    AudioSink,
}

struct GraphViewer;

impl SnarlViewer<Node> for GraphViewer {
    fn title(&mut self, node: &Node) -> String {
        match node {
            Node::FileSource { .. } => "File Source".into(),
            Node::Gain(_) => "Gain".into(),
            Node::AudioSink => "Audio Sink".into(),
        }
    }

    fn inputs(&mut self, node: &Node) -> usize {
        match node {
            Node::FileSource { .. } => 0,
            Node::Gain(_) | Node::AudioSink => 1,
        }
    }

    fn outputs(&mut self, node: &Node) -> usize {
        match node {
            Node::FileSource { .. } | Node::Gain(_) => 1,
            Node::AudioSink => 0,
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
            Node::Gain(gain) => {
                ui.add(DragValue::new(gain).speed(0.01).prefix("Gain: "));
            }
            Node::AudioSink => {}
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
            snarl.insert_node(pos, Node::Gain(1.0));
            ui.close();
        }

        if ui.button("Audio Sink").clicked() {
            snarl.insert_node(pos, Node::AudioSink);
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
}

impl App for GhostAnalyzer {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        CentralPanel::default()
            .frame(Frame::central_panel(ui.style()).fill(Color32::from_rgb(9, 12, 18)))
            .show(ui, |ui| {
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
            }))
        }),
    )
}
