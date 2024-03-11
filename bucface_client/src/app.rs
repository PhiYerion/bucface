use bucface_utils::{Event, EventDBResponse};

use crate::net::ws_client::{WebSocketError, WsClient};
use crate::ui::main_window::body;

pub struct State<'a> {
    pub author: &'a str,
    pub machine: &'a str,
}

pub struct App<'a> {
    pub logs: Vec<EventDBResponse>,
    pub log_buf: String,
    pub ws_client: WsClient,
    pub state: State<'a>,
}

impl App<'_> {
    pub fn new(ws_client: WsClient) -> Self {
        App {
            logs: Vec::new(),
            log_buf: String::new(),
            ws_client,
            state: State {
                author: "Anonymous",
                machine: "Unknown",
            },
        }
    }

    pub fn get_logs(&mut self) {
        log::debug!("Getting logs");
        let count = self.ws_client.get_buf_logs(&mut self.logs);
        log::info!("Got {count} new logs");
    }

    pub fn send_log(&mut self) -> Result<(), WebSocketError> {
        let event = self.create_event_from_buf();
        self.ws_client.send_log(event)?;
        self.log_buf.clear();

        Ok(())
    }

    pub fn create_event_from_buf(&mut self) -> Event {
        bucface_utils::Event {
            time: chrono::Utc::now().naive_utc(),
            author: self.state.author.into(),
            event: self.log_buf.clone(),
            machine: self.state.machine.into(),
        }
    }
}

impl eframe::App for App<'_> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            body(ui, self);
        });
        self.get_logs();
    }
}
