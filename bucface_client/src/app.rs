use crate::net::sync::{get_events, send_event, SendEventError, UpdateLogsError};
use crate::ui::main_window::body;
use bucface_utils::Event;
use parking_lot::lock_api::RwLock;
use parking_lot::RawRwLock;
use tokio::task::JoinHandle;
use std::future::Future;
use std::sync::Arc;

#[derive(Debug, Default)]
pub struct App {
    pub logs: Vec<Event>,
    pub log_buf: String,
    pub new_logs_buf: Arc<RwLock<RawRwLock, Vec<Event>>>,
    pub send_thread: Option<tokio::task::JoinHandle<Result<(), UpdateLogsError>>>,
    pub reqwest_client: Arc<reqwest::Client>,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            body(ui, self);
        });

        if self.send_thread.is_none() || self.send_thread.as_ref().is_some_and(|t| t.is_finished()) {
            let client = self.reqwest_client.clone();
            let new_logs_buf = self.new_logs_buf.clone();
            let index = self.logs.len() + new_logs_buf.read().len();
            log::debug!("Getting new logs starting at index {}", index);
            self.send_thread = Some(tokio::spawn(async move {
                let result = get_events("http://127.0.0.1:8080/events".to_string(), &client, index).await;

                match result {
                    Ok(mut logs) => {
                        new_logs_buf.write().append(&mut logs);
                        log::debug!("New logs: {:?}", logs);
                        Ok(())
                    }
                    Err(e) => {
                        log::error!("Error getting new logs: {:?}", e);
                        Err(e)
                    }
                }
            }));
        }
    }
}

impl App {
    pub fn get_new_logs(&mut self) {
        if let Some(mut new_logs) = self.new_logs_buf.try_write() {
            self.logs.append(&mut new_logs);
            log::info!("New logs: {:?}", self.logs);
            new_logs.clear();
        }
    }

    pub fn send_logs(&mut self) -> Option<JoinHandle<Result<(), SendEventError>>> {
        if self.log_buf.is_empty() {
            return None;
        }

        log::info!("Sending logs {}", self.log_buf);

        let mut logs = String::new();
        std::mem::swap(&mut self.log_buf, &mut logs);

        let handle = tokio::spawn(async move {
            send_log_entry(&logs).await
        });
        self.log_buf.clear();

        Some(handle)
    }
}

fn send_log_entry(
    log_buf: &str,
) -> impl Future<Output = Result<(), SendEventError>> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_send_logs() {
        println!("Make sure the server is running...");

        let mut app = App {
            log_buf : "test".into(),
            ..Default::default()
        };
        let handle = app.send_logs().expect("log_buf is empty when it should not be");
        assert!(handle.await.is_ok());

        assert_eq!(app.log_buf, "");
    }
}
