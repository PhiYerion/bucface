use bucface_utils::ws::WsStream;
use bucface_utils::{ClientMessage, Event, EventDBResponse};
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite;

use super::ws_receiver::start_receiver;
use super::ws_sender::start_sender;

#[derive(Debug)]
pub enum ConnectionError {
    NoResponse,
    InvalidResponse,
    IOError(tungstenite::Error),
}

#[derive(Debug)]
pub enum WebSocketError {
    Connection(ConnectionError),
    Soft(String),
    Hard(String),
}

#[derive(Debug)]
pub struct WsClient {
    pub receiver: Receiver,
    pub sender: Sender,
}

#[derive(Debug)]
pub struct Receiver {
    /// The thread that is taking and handling the responses from the server
    pub receiver: tokio::task::JoinHandle<()>,
    /// A channel to read from to get the responses from the server
    pub rx: tokio::sync::mpsc::Receiver<EventDBResponse>,
}

#[derive(Debug)]
pub struct Sender {
    /// The thread that is sending the logs to the server
    pub sender: tokio::task::JoinHandle<()>,
    /// The channel to send events to, which are then sent to the server
    pub tx: tokio::sync::mpsc::Sender<ClientMessage>,
}

impl WsClient {
    pub async fn new(dest: &str) -> Result<WsClient, WebSocketError> {
        let stream = connect(dest).await?;

        let (mut write, mut read) = stream.split();

        let (receiver_tx, receiver_rx) = tokio::sync::mpsc::channel::<EventDBResponse>(128);
        let receiver = tokio::spawn(async move {
            match start_receiver(&mut read, &receiver_tx).await {
                Ok(_) => log::info!("WebSocket connection closed"),
                Err(e) => log::error!("Error handling WebSocket connection: {:?}", e),
            }
        });

        let (sender_tx, mut sender_rx) = tokio::sync::mpsc::channel::<ClientMessage>(128);
        let sender = tokio::spawn(async move {
            start_sender(&mut write, &mut sender_rx).await;
        });

        Ok(WsClient {
            receiver: Receiver {
                receiver,
                rx: receiver_rx,
            },
            sender: Sender {
                sender,
                tx: sender_tx,
            },
        })
    }

    pub fn send_log(&mut self, log: Event) -> Result<(), WebSocketError> {
        let message = ClientMessage::NewEvent(log);
        self.sender.tx.try_send(message).map_err(|e| {
            log::error!("Error sending log: {:?}", e);
            WebSocketError::Soft("Error sending log".into())
        })
    }

    pub fn get_log(&mut self, id: u64) -> Result<(), WebSocketError> {
        self.sender
            .tx
            .try_send(ClientMessage::GetEvent(id))
            .map_err(|e| {
                log::error!("Error getting log: {:?}", e);
                WebSocketError::Soft("Error sending get log request".into())
            })
    }

    pub fn get_buf_logs(&mut self, buf: &mut Vec<EventDBResponse>) -> Vec<u64> {
        let mut new_log_ids: Vec<u64> = Vec::new();
        while let Ok(log) = self.receiver.rx.try_recv() {
            new_log_ids.push(log.id);
            buf.push(log);
        }

        new_log_ids
    }

    /// Gets all logs including and after the given id
    pub fn get_logs_since(&mut self, since_id: u64) -> Result<(), WebSocketError> {
        let message = ClientMessage::GetSince(since_id);
        self.sender.tx.try_send(message).map_err(|e| {
            log::error!("Error sending get logs since request: {:?}", e);
            WebSocketError::Soft("Error sending get logs since request".into())
        })
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
        WebSocketError::Hard(format!("Error connecting to {}: {:?}", dest, e))
    })?;

    verify_conn(&mut stream).await.map_err(|e| {
        log::error!("Error verifying connection: {:?}", e);
        WebSocketError::Connection(e)
    })?;
    log::trace!("Connected to {}", dest);

    Ok(stream)
}
