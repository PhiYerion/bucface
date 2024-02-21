use crate::net::sync::{get_events, send_event};
use bucface_utils::Event;
use parking_lot::lock_api::RwLock;
use parking_lot::RawRwLock;
use std::future::Future;
use std::sync::Arc;

#[derive(Debug, Default)]
pub struct App {
    logs: Vec<Event>,
    log_buf: String,
    new_events: Arc<RwLock<RawRwLock, Vec<Event>>>,
    send_thread: Option<tokio::task::JoinHandle<()>>,
    reqwest_client: Arc<reqwest::Client>,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            body(ui, self);
        });

        if self.send_thread.is_none() || self.send_thread.as_ref().is_some_and(|t| t.is_finished()) {
            let client = self.reqwest_client.clone();
            let new_logs = self.new_events.clone();
            let index = self.logs.len() + new_logs.read().len();
            log::info!("Starting new send thread at index {}", index);
            self.send_thread = Some(tokio::spawn(async move {
                let result = get_events("http://127.0.0.1:8080/events".to_string(), &client, index).await;
                if let Ok(mut logs) = result {
                    new_logs.write().append(&mut logs);
                }
                log::info!("Send thread finished");
            }));
        }
    }
}

impl App {
    fn get_new_logs(&mut self) {
        if let Some(mut new_logs) = self.new_events.try_write() {
            self.logs.append(&mut new_logs);
            log::info!("New logs: {:?}", self.logs);
            new_logs.clear();
        } else {
            log::error!("Failed to acquire write lock on new_events");
        }
    }

    fn send_logs(&mut self) {
        if self.log_buf.is_empty() {
            return;
        }

        log::info!("Sending logs {}", self.log_buf);

        let mut logs = String::new();
        std::mem::swap(&mut self.log_buf, &mut logs);

        tokio::spawn(async move {
            send_log_entry(&logs).await.unwrap();
        });
        self.log_buf.clear();
    }
}

fn header(ui: &mut egui::Ui) {
    ui.heading("BucFace Client v0.1");
}

fn log_entry(ui: &mut egui::Ui, app: &mut App) {
    ui.vertical(|ui| {
        ui.label("Log");
        ui.text_edit_multiline(&mut app.log_buf);
        if ui.button("Send Log").clicked() {
            app.send_logs();
        }
    });
}

fn log_panel(ui: &mut egui::Ui, app: &mut App) {
    ui.vertical(|ui| {
        // create vertical collumn of all logs from App::logs
        ui.label("Logs");
        if ui.button("Refresh").clicked() {
            app.get_new_logs();
        }
        for log in &app.logs {
            ui.label(&*log.event.clone());
        }
    });
}

fn body(ui: &mut egui::Ui, app: &mut App) {
    ui.vertical(|ui| {
        header(ui);
        ui.horizontal(|ui| {
            log_entry(ui, app);
            log_panel(ui, app);
        })
    });
}

fn send_log_entry(
    log_buf: &str,
) -> impl Future<Output = Result<(), crate::net::sync::SendEventError>> {
    let event = bucface_utils::Event {
        time: chrono::Utc::now().naive_utc(),
        author: "test".into(),
        event: log_buf.into(),
        machine: "test".into(),
    };

    let client = Arc::new(reqwest::Client::new());

    send_event(
        event,
        "http://127.0.0.1:8080/events/create",
        client.clone(),
        10,
    )
}
