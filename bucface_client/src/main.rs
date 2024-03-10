use crate::net::ws_client;

mod app;
mod net;

#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    env_logger::init();
    let mut ws_client = ws_client::start("ws://127.0.0.1:8080").await.unwrap();
    log::trace!("Connected to {}", "127.0.0.1:8080");

    while let Some(msg) = ws_client.receiver.rx.recv().await {
        log::trace!("Received: {msg:?}");
    }

    Ok(())
    /* let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default(),
        ..Default::default()
    };

    eframe::run_native(
        "Confirm exit",
        options,
        Box::new(|_cc| Box::<App>::default()),
    ) */
}
