use bucface_utils::Event;

use crate::net::ws_client::WsClient;

pub struct App {
    logs: Vec<Event>,
    log_buf: String,
    ws_client: WsClient,
}

impl App {
    pub fn new(ws_client: WsClient) -> Self {
        App {
            logs: Vec::new(),
            log_buf: String::new(),
            ws_client,
        }
    }
}


impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            body(ui, self);
        });
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
        for log in app.logs.iter() {
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

fn create_log(
    log: &str,
    author: &str,
    machine: &str,
) -> Event {
    bucface_utils::Event {
        time: chrono::Utc::now().naive_utc(),
        author: author.into(),
        event: log.into(),
        machine: machine.into(),
    }
}
