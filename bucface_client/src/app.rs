use crate::net::sync::send_event;
use bucface_utils::Events;
use parking_lot::lock_api::RwLock;
use parking_lot::RawRwLock;
use std::future::Future;
use std::sync::Arc;

#[derive(Default)]
pub struct App {
    logs: Events,
    log_buf: String,
    new_events: Arc<RwLock<RawRwLock, Events>>,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            body(ui, self);
        });
    }
}

impl App {
    fn get_new_logs(&mut self) -> Option<()> {
        if !self.new_events.try_read()?.inner.is_empty() {
            self.logs
                .inner
                .extend_from_slice(&self.new_events.try_write()?.inner);
        }

        Some(())
    }
}

fn header(ui: &mut egui::Ui) {
    ui.heading("BucFace Client v0.1");
}

fn log_entry(ui: &mut egui::Ui, buf: &mut String) {
    ui.vertical(|ui| {
        ui.label("Log");
        ui.text_edit_multiline(buf);
    });
}

fn log_panel(ui: &mut egui::Ui, app: &App) {
    ui.vertical(|ui| {
        // create vertical collumn of all logs from App::logs
        for log in app.logs.inner.iter() {
            ui.label(&*log.event.clone());
        }
    });
}

fn body(ui: &mut egui::Ui, app: &mut App) {
    ui.vertical(|ui| {
        header(ui);
        ui.horizontal(|ui| {
            log_entry(ui, &mut app.log_buf);
            log_panel(ui, app);
        })
    });
}

fn send_log_entry<'a>(
    log: &str,
    author: &str,
    machine: &str,
    dest: &'a str,
) -> impl Future<Output = Result<(), crate::net::sync::SendEventError>> + 'a {
    let event = bucface_utils::Event {
        time: chrono::Utc::now().naive_utc(),
        author: author.into(),
        event: log.into(),
        machine: machine.into(),
    };

    let client = Arc::new(reqwest::Client::new());

    send_event(event, dest, client.clone(), 10)
}
