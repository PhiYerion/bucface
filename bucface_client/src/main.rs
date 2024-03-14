#![feature(let_chains)]
#![feature(async_closure)]
mod app;
mod net;
mod ui;

use app::App;

fn main() -> Result<(), eframe::Error> {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default(),
        ..Default::default()
    };

    let app = App::new();

    eframe::run_native("Confirm exit", options, Box::new(|_cc| Box::new(app)))
}
