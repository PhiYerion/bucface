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
    pub log_ids: Vec<u64>,
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
            log_ids: Vec::new(),
        }
    }

    pub fn send_log(&mut self) -> Result<(), WebSocketError> {
        let event = self.create_event_from_buf();
        self.ws_client.send_log(event)?;
        self.log_buf.clear();

        Ok(())
    }

    // This function would be faster if this is either inserting is handled by
    // ws_client.get_buf_logs or if a function is passed to ws_client that inserts
    // the logs into the log_ids vector.
    pub fn get_logs(&mut self) {
        self.ws_client.get_buf_logs(&mut self.logs)
            .drain(..)
            .for_each(|id| {
                match self.log_ids.binary_search(&id) {
                    Ok(_) => log::warn!("Attempting to insert log {id}, but we already have it"),
                    Err(i) => self.log_ids.insert(i, id),
                }
            });
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
