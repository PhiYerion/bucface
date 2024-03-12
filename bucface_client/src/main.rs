#![feature(let_chains)]
mod app;
mod net;
mod ui;
use std::sync::Arc;

use app::App;
use parking_lot::Mutex;

use self::net::ws_client::WsClient;

#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default(),
        ..Default::default()
    };

    let context = Arc::new(Mutex::new(None::<egui::Context>));
    let ws_client = WsClient::new("ws://localhost:8080", context.clone()).await.unwrap();
    let app = App::new(ws_client, context);

    eframe::run_native("Confirm exit", options, Box::new(|_cc| Box::new(app)))
}
