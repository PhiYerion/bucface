use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use bucface_utils::{Event, Events};
use reqwest::StatusCode;
use rmp_serde::Serializer;
use serde::Serialize;
use tokio::sync::Mutex;

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

#[derive(Debug)]
pub struct App<'a> {
    pub events_len: AtomicUsize,
    pub events: std::sync::Mutex<Vec<Event>>,
    pub machines: Vec<&'a str>,
    pub name: std::sync::Mutex<Box<str>>,
    pub buf: std::sync::Mutex<Vec<u8>>,
    pub mode: std::sync::Mutex<AppMode>,
    client: reqwest::Client,
}

impl Default for App<'_> {
    fn default() -> Self {
        Self {
            events_len: AtomicUsize::new(0),
            events: std::sync::Mutex::new(Vec::new()),
            machines: Vec::new(),
            name: std::sync::Mutex::new(Box::from("")),
            buf: Vec::new().into(),
            mode: std::sync::Mutex::new(AppMode::Entry),
            client: reqwest::Client::new(),
        }
    }
}

#[derive(Debug)]
pub enum UpdateLogsError {
    Reqwest(reqwest::Error),
    Rmp(rmp_serde::decode::Error),
    InvalidStatusCode(StatusCode),
    NoChange,
}

impl App<'_> {
    pub(crate) async fn send_buf(&self) -> std::io::Result<()> {
        let mode = *self.mode.lock().unwrap();
        *self.mode.lock().unwrap() = AppMode::Normal;

        match mode {
            AppMode::Entry => self.buf_to_name(),
            AppMode::Logging => self.send_buf_as_log().await,
            AppMode::Quitting | AppMode::Normal => Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "Attempting to send buffer while quitting",
            )),
        }
    }

    async fn send_buf_as_log(&self) -> std::io::Result<()> {
        let log = Event {
            author: self.name.lock().unwrap().clone().into(),
            machine: "test".into(),
            event: self
                .buf.lock().unwrap()
                .iter()
                .map(|x| char::from(*x))
                .collect::<String>()
                .into(),
            time: chrono::Utc::now().naive_utc(),
        };
        let logs = Events {
            inner: vec![log],
        };
        self.buf.lock().unwrap().clear();
        let mut buf = Vec::new();

        logs.serialize(&mut Serializer::new(&mut buf))
            .map_err(|_| {
                std::io::Error::new(std::io::ErrorKind::Other, "Failed to serialize log")
            })?;

        const TRY_MAX: usize = 10;
        let mut counter = 0;
        while let Err(e) = self
            .client
            .post("http://127.0.0.1:8080/events")
            .body(buf.clone())
            .send()
            .await
        {
            counter += 1;
            if counter > TRY_MAX {
                return Err(std::io::Error::new(std::io::ErrorKind::Other, e));
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }

        Ok(())
    }

    async fn update_logs(&self) -> Result<(), UpdateLogsError> {
        let res = self
            .client
            .get("http://127.0.0.1:8080/events".to_string() + &self.events_len.load(Ordering::Acquire).to_string())
            .send()
            .await
            .map_err(UpdateLogsError::Reqwest)?;
        if !res.status().is_success() {
            return Err(UpdateLogsError::InvalidStatusCode(res.status()));
        }
        let bytes = res.bytes().await.map_err(UpdateLogsError::Reqwest)?;
        let new_events: Events = rmp_serde::from_slice(&bytes).map_err(UpdateLogsError::Rmp)?;
        self.events.lock().unwrap().extend_from_slice(&new_events.inner);
        self.events_len.store(self.events.lock().unwrap().len(), Ordering::Release);

        Ok(())
    }

    pub async fn update(&self) -> Result<(), UpdateLogsError> {
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

    fn buf_to_name(&self) -> std::io::Result<()> {
        *self.name.lock().unwrap() = self.buf.lock().unwrap().iter().map(|x| char::from(*x)).collect::<String>().into();
        self.buf.lock().unwrap().clear();
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
        app.buf.lock().unwrap().extend(message);
        app.send_buf_as_log().await.unwrap();
    }
}
