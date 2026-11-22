use eframe::egui::{CentralPanel, Color32, Frame, Ui, Visuals};
use eframe::{App, NativeOptions};

struct GhostAnalyzer;

impl App for GhostAnalyzer {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        CentralPanel::default()
            .frame(Frame::central_panel(ui.style()).fill(Color32::from_rgb(9, 12, 18)))
            .show(ui, |_ui| {});
    }
}

fn main() -> eframe::Result {
    eframe::run_native(
        "GhostAnalyzer",
        NativeOptions::default(),
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(Visuals::dark());
            Ok(Box::new(GhostAnalyzer))
        }),
    )
}
