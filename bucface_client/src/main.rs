mod app;
mod net;
mod ui;
use app::App;

use self::net::ws_client::WsClient;

#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default(),
        ..Default::default()
    };

    let ws_client = WsClient::new("ws://localhost:8080").await.unwrap();
    let app = App::new(ws_client);

    eframe::run_native("Confirm exit", options, Box::new(|_cc| Box::new(app)))
}
