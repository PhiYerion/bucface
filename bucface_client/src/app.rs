use bucface_utils::{Event, EventDB, EventDBErrorSerde, ServerResponse};
use tokio::runtime::Runtime;

use crate::net::ws_client::{WebSocketError, WebSocketStatus, WsClient};
use crate::ui::main_window::body;

pub struct State<'a> {
    pub author: &'a str,
    pub machine: &'a str,
}

pub struct App<'a> {
    pub state: State<'a>,
    pub logs: Vec<EventDB>,
    pub log_ids: Vec<u64>,
    pub runtime: Runtime,
    pub ws_client: WebSocketStatus,
    pub bufs: AppBufs,
}

pub struct AppBufs {
    pub context: Option<egui::Context>,
    pub log: String,
    pub server: String,
    pub port: String,
}

impl App<'_> {
    pub fn new() -> Self {
        App {
            runtime: Runtime::new().unwrap(),
            logs: Vec::new(),
            ws_client: WebSocketStatus::Disconnected,
            state: State {
                author: "Anonymous",
                machine: "Unknown",
            },
            log_ids: Vec::new(),
            bufs: AppBufs {
                context: None,
                log: String::new(),
                server: String::from("localhost"),
                port: String::from("8080"),
            },
        }
    }

    pub fn send_log(&mut self) -> Result<(), WebSocketError> {
        if let WebSocketStatus::Connected(ws_client) = &self.ws_client {
            let event = self.create_event_from_buf();
            ws_client.send_log(event).map_err(WebSocketError::SendError)?;
            self.bufs.log.clear();
            Ok(())
        } else {
            Err(WebSocketError::DoesNotExist)
        }
    }

    pub fn get_missing_logs(&mut self) -> Result<usize, WebSocketError> {
        if let WebSocketStatus::Connected(ws_client) = &self.ws_client {
            // id starts at 0, so we -1
            if self.logs.is_empty()
                || self.log_ids.len() - 1 == self.log_ids[self.log_ids.len() - 1] as usize
            {
                return Ok(0);
            }

            log::debug!("There are {} logs in the logs vector", self.log_ids.len());
            log::debug!(
                "The last log id is {}",
                self.log_ids[self.log_ids.len() - 1]
            );
            log::debug!("{:?}", self.log_ids);

            let missing_amt = self.log_ids[self.log_ids.len() - 1] as usize - (self.log_ids.len() - 1);
            log::debug!("There are {missing_amt} logs missing");

            let mut id_cursor = 0;
            let mut counter = 0;
            for i in 0..self.log_ids.len() {
                if self.log_ids[i] != id_cursor {
                    for id in id_cursor..self.log_ids[i] {
                        log::debug!("Getting missing log {id}");
                        ws_client.get_log(id).map_err(WebSocketError::SendError)?;

                        counter += 1;
                    }
                }
                id_cursor += self.log_ids[i] + 1;
            }

            Ok(missing_amt)
        } else {
            Err(WebSocketError::DoesNotExist)
        }
    }

    pub fn get_logs(&mut self) -> Result<(), WebSocketError> {
        if let WebSocketStatus::Connected(ws_client) = &mut self.ws_client {
            ws_client.get_buf_logs(|response| {
                match response {
                    ServerResponse::Event(event) => {
                        let id = event._id;
                        match self.log_ids.binary_search(&id) {
                            Ok(_) => {
                                log::warn!("Attempting to insert log {id}, but we already have it")
                            }
                            Err(i) => {
                                self.log_ids.insert(i, id);
                                self.logs.insert(i, event);
                            }
                        }
                    }
                    ServerResponse::Error(error) => {
                        log::error!("Error getting buf logs: {error:?}");
                        return Some(error);
                    }
                    _ => {}
                }

                None::<EventDBErrorSerde>
            });

            assert_eq!(self.log_ids.len(), self.logs.len());
            Ok(())
        } else {
            Err(WebSocketError::DoesNotExist)
        }
    }

    pub fn create_event_from_buf(&self) -> Event {
        bucface_utils::Event {
            time: chrono::Utc::now().naive_utc(),
            author: self.state.author.into(),
            event: self.bufs.log.clone(),
            machine: self.state.machine.into(),
        }
    }

    pub fn clear_logs(&mut self) {
        self.logs.clear();
        self.log_ids.clear();
    }

    pub fn set_endpoint(&mut self, context: &egui::Context) {
        let new_endpoint = format!(
            "ws://{server}:{port}",
            server = self.bufs.server,
            port = self.bufs.port
        );
        let context = context.clone();
        let result = self
            .runtime
            .block_on(WsClient::new(&new_endpoint, context));
        match result {
            Ok(ws_client) => {
                self.ws_client = WebSocketStatus::Connected(ws_client);
            }
            Err(e) => {
                log::error!("Error connecting to endpoint: {:?}", e);
                self.ws_client = WebSocketStatus::Disconnected;
            }
        }
    }
}

impl eframe::App for App<'_> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let start = std::time::Instant::now();
        egui::CentralPanel::default().show(ctx, |ui| {
            body(ui, ctx, self);
        });
        log::trace!("update took: {}ns", start.elapsed().as_nanos());
        let update_end = std::time::Instant::now();
        let _ = self.get_logs();
        log::trace!("get_logs took: {}ns", update_end.elapsed().as_nanos());
        let get_logs_end = std::time::Instant::now();
        self.get_missing_logs();
        log::trace!(
            "get_missing_logs took: {}ns",
            get_logs_end.elapsed().as_nanos()
        );
    }
}
