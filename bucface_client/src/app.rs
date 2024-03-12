use std::sync::Arc;

use bucface_utils::{Event, EventDB, EventDBErrorSerde, ServerResponse};
use parking_lot::Mutex;

use crate::net::ws_client::{WebSocketError, WsClient};
use crate::ui::main_window::body;

pub struct State<'a> {
    pub author: &'a str,
    pub machine: &'a str,
}

pub struct App<'a> {
    pub logs: Vec<EventDB>,
    pub log_buf: String,
    pub ws_client: WsClient,
    pub state: State<'a>,
    pub log_ids: Vec<u64>,
    pub egui_ctx: Arc<Mutex<Option<egui::Context>>>,
}

impl App<'_> {
    pub fn new(ws_client: WsClient, egui_ctx: Arc<Mutex<Option<egui::Context>>>) -> Self {
        App {
            logs: Vec::new(),
            log_buf: String::new(),
            ws_client,
            state: State {
                author: "Anonymous",
                machine: "Unknown",
            },
            log_ids: Vec::new(),
            egui_ctx,
        }
    }

    pub fn send_log(&mut self) -> Result<(), WebSocketError> {
        let event = self.create_event_from_buf();
        self.ws_client.send_log(event)?;
        self.log_buf.clear();

        Ok(())
    }

    pub fn get_missing_logs(&mut self) -> usize {
        // id starts at 0, so we -1
        if self.logs.is_empty()
            || self.log_ids.len() - 1 == self.log_ids[self.log_ids.len() - 1] as usize
        {
            return 0;
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
                    let _ = self
                        .ws_client
                        .get_log(id)
                        .map_err(|e| log::error!("Error getting missing log: {e:?}"));

                    counter += 1;
                }
            }
            id_cursor += self.log_ids[i] + 1;
        }

        assert_eq!(counter, missing_amt);
        missing_amt
    }

    pub fn get_logs(&mut self) -> Result<(), EventDBErrorSerde> {
        self.ws_client.get_buf_logs(|response| {
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
        if let Some(mut egui_ctx) = self.egui_ctx.try_lock()
            && egui_ctx.is_none()
        {
            *egui_ctx = Some(ctx.clone());
        }

        let start = std::time::Instant::now();
        egui::CentralPanel::default().show(ctx, |ui| {
            body(ui, self);
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
