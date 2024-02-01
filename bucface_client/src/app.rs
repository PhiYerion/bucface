use std::future::{Future, IntoFuture};
use std::pin::{pin, Pin};

use bucface_utils::{Event, Events};
use reqwest::StatusCode;
use tokio::task::JoinHandle;

use crate::net::sync::{get_events, send_event, SendEventError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Entry,
    Normal,
    Logging,
    Quitting,
}

impl std::fmt::Display for AppMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppMode::Entry => write!(f, "Entry"),
            AppMode::Normal => write!(f, "Normal"),
            AppMode::Logging => write!(f, "Logging"),
            AppMode::Quitting => write!(f, "Quitting"),
        }
    }
}

impl Default for AppMode {
    fn default() -> Self {
        Self::Entry
    }
}

#[derive(Debug, Default, Clone)]
pub struct App<'a> {
    pub events: Vec<Event>,
    pub machines: Vec<&'a str>,
    pub name: Box<str>,
    pub buf: Vec<u8>,
    pub mode: AppMode,
    client: reqwest::Client,
}

#[derive(Debug)]
pub enum UpdateLogsError {
    Reqwest(reqwest::Error),
    Rmp(rmp_serde::decode::Error),
    InvalidStatusCode(StatusCode),
    NoChange,
}

impl App<'_> {
    pub(crate) fn send_buf(&mut self) -> Option<impl Future> {
        let mode = self.mode;
        self.mode = AppMode::Normal;

        match mode {
            AppMode::Entry => {
                self.buf_to_name();
                None
            }
            AppMode::Logging => Some(self.send_buf_as_log()),
            AppMode::Quitting | AppMode::Normal => { None },
        }
    }

    fn send_buf_as_log(&mut self) -> impl Future {
        let log = Event {
            author: self.name.clone(),
            machine: "test".into(),
            event: self
                .buf
                .iter()
                .map(|x| char::from(*x))
                .collect::<String>()
                .into(),
            time: chrono::Utc::now().naive_utc(),
        };
        self.buf.clear();

        let client = self.client.clone();
        tokio::spawn(
            async move { send_event(log, "http://127.0.0.1:8080/events/create", &client, 10).await }
        )
    }

    pub fn update_logs(&self) -> impl Future<Output = Result<Events, UpdateLogsError>> {
        let client = self.client.clone();
        let len = self.events.len();

        async move { get_events("http://127.0.0.1:8080/events".to_string(), client, len).await }
    }

    /* pub async fn update(&self) -> Result<(), UpdateLogsError> {
            const TRY_MAX: usize = 10;
            let mut counter = 0;
            while let Err(e) = self.update_logs().await {
                counter += 1;
                if counter > TRY_MAX {
                    return Err(e);
                }
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }

            Ok(())
        }
    */
    fn buf_to_name(&mut self) -> std::io::Result<()> {
        self.name = self
            .buf
            .iter()
            .map(|x| char::from(*x))
            .collect::<String>()
            .into();
        self.buf.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn send_buf_as_log() {
        let mut app = App {
            mode: AppMode::Logging.into(),
            ..Default::default()
        };
        let message = "test message".bytes();
        app.buf.extend(message);
        app.send_buf_as_log().await;
    }
}
