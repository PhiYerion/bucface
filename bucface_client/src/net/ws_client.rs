use bucface_utils::ws::{WsStream, WsWriter};
use bucface_utils::Event;
use futures_util::stream::SplitSink;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite;

use super::ws_receiver::handle_connection;
use super::ws_sender::send_log;

#[derive(Debug)]
pub enum ConnectionError {
    NoResponse,
    InvalidResponse,
    IOError(tungstenite::Error),
}

#[derive(Debug)]
pub enum WebSocketError {
    ConnectionError(ConnectionError),
    SoftError(String),
    HardError(String),
}

#[derive(Debug)]
pub struct WsClient {
    pub receiver: Receiver,
    pub write: WsWriter,
}

#[derive(Debug)]
pub struct Receiver {
    pub receiver: tokio::task::JoinHandle<()>,
    pub rx: tokio::sync::mpsc::Receiver<Event>,
}

impl WsClient {
    pub async fn new(dest: &str) -> Result<WsClient, WebSocketError> {
        let stream = connect(dest).await?;

        let (write, mut read) = stream.split();

        let (receiver_tx, receiver_rx) = tokio::sync::mpsc::channel::<Event>(128);
        let receiver = tokio::spawn(async move {
            match handle_connection(&mut read, &receiver_tx).await {
                Ok(_) => log::info!("WebSocket connection closed"),
                Err(e) => log::error!("Error handling WebSocket connection: {:?}", e),
            }
        });

        Ok(WsClient {
            receiver: Receiver {
                receiver,
                rx: receiver_rx,
            },
            write,
        })
    }

    pub async fn send_log(&mut self, log: Event) -> Result<(), WebSocketError> {
        send_log(log, &mut self.write).await.map_err(|e| {
            log::error!("Error sending log: {:?}", e);
            WebSocketError::SoftError(format!("Error sending log: {:?}", e))
        })
    }

    pub async fn get_logs(&mut self, buf: &mut Vec<Event>) {
        while let Some(log) = self.receiver.rx.recv().await {
            buf.push(log);
        }
    }
}

pub async fn verify_conn(stream: &mut WsStream) -> Result<(), ConnectionError> {
    log::debug!("Verifying WebSocket connection");

    const ECHO: &[u8] = b"echo\n";
    let (mut writer, mut reader) = stream.split();

    log::trace!("Sending ping");
    writer
        .send(tungstenite::Message::Ping(ECHO.to_vec()))
        .await
        .map_err(ConnectionError::IOError)?;

    while let Some(msg) = reader.next().await {
        log::trace!("Received message: {:?}", msg);
        let msg = msg.map_err(ConnectionError::IOError)?;
        if msg.is_pong() {
            let data = msg.into_data();
            if data != ECHO {
                log::warn!("Invalid pong response, treating as non-fatal: {data:?}");
                return Err(ConnectionError::InvalidResponse);
            }

            log::debug!("WebSocket connection verified");
            return Ok(());
        }
    }

    log::warn!("No response to ping");
    Err(ConnectionError::NoResponse)
}

async fn connect(dest: &str) -> Result<WsStream, WebSocketError> {
    log::info!("Connecting to {}", dest);
    let (mut stream, _) = tokio_tungstenite::connect_async(dest).await.map_err(|e| {
        log::error!("Error connecting to {}: {:?}", dest, e);
        WebSocketError::HardError(format!("Error connecting to {}: {:?}", dest, e))
    })?;

    verify_conn(&mut stream).await.map_err(|e| {
        log::error!("Error verifying connection: {:?}", e);
        WebSocketError::ConnectionError(e)
    })?;
    log::trace!("Connected to {}", dest);

    Ok(stream)
}
