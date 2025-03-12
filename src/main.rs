mod app;
mod config;
mod hotkeys;
mod tracker;

use app::MouseMinderApp;
use eframe::{Frame, NativeOptions, egui};

impl eframe::App for MouseMinderApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        self.update(ctx);
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 500.0])
            .with_resizable(true),
        ..Default::default()
    };

    eframe::run_native(
        config::APP_NAME,
        options,
        Box::new(|cc| Ok(Box::new(MouseMinderApp::new(&cc.egui_ctx)))),
    )
}
