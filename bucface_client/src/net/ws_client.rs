use bucface_utils::ws::WsStream;
use bucface_utils::{ClientMessage, Event, EventDBErrorSerde, ServerResponse};
use egui::Context;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc::error::TrySendError;
use tokio_tungstenite::tungstenite;
use url::Url;

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
    UrlParseError(url::ParseError),
    Connection(ConnectionError),
    NotWsUrl,
    DoesNotExist,
    SendError(TrySendError<ClientMessage>),
    ServerError(EventDBErrorSerde),
}

#[derive(Debug)]
pub enum WebSocketStatus {
    Connected(WsClient),
    Error(WebSocketError),
    Disconnected,
}

impl ToString for WebSocketStatus {
    fn to_string(&self) -> String {
        match self {
            WebSocketStatus::Connected(_) => "Connected".to_string(),
            WebSocketStatus::Error(e) => format!("Error: {:?}", e),
            WebSocketStatus::Disconnected => "Disconnected".to_string(),
        }
    }
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
    pub rx: tokio::sync::mpsc::Receiver<ServerResponse>,
}

#[derive(Debug)]
pub struct Sender {
    /// The thread that is sending the logs to the server
    pub sender: tokio::task::JoinHandle<()>,
    /// The channel to send events to, which are then sent to the server
    pub tx: tokio::sync::mpsc::Sender<ClientMessage>,
}

impl WsClient {
    pub async fn new(dest: &str, egui_ctx: Context) -> Result<WsClient, WebSocketError> {
        let url = dest
            .parse::<url::Url>()
            .map_err(WebSocketError::UrlParseError)?;
        if url.scheme() != "ws" {
            return Err(WebSocketError::NotWsUrl);
        }

        let stream = connect(url).await?;

        let (mut write, mut read) = stream.split();

        let (receiver_tx, receiver_rx) = tokio::sync::mpsc::channel::<ServerResponse>(128);
        let receiver = tokio::spawn(async move {
            match start_receiver(&mut read, &receiver_tx, egui_ctx).await {
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

    pub fn send_log(&self, log: Event) -> Result<(), TrySendError<ClientMessage>> {
        let message = ClientMessage::NewEvent(log);

        self.sender.tx.try_send(message)
    }

    pub fn get_log(&self, id: u64) -> Result<(), TrySendError<ClientMessage>> {
        self.sender
            .tx
            .try_send(ClientMessage::GetEvent(id))
    }

    pub fn get_buf_logs<T>(
        &mut self,
        mut post: impl FnMut(ServerResponse) -> Option<T>,
    ) -> Option<T> {
        while let Ok(res) = self.receiver.rx.try_recv() {
            if let Some(ret) = post(res) {
                return Some(ret);
            }
        }

        None
    }

    /// Gets all logs including and after the given id
    pub fn get_logs_since(&self, since_id: u64) -> Result<(), TrySendError<ClientMessage>> {
        let message = ClientMessage::GetSince(since_id);

        self.sender.tx.try_send(message)
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

async fn connect(url: Url) -> Result<WsStream, WebSocketError> {
    log::debug!("Connecting to {}", url);

    let (mut stream, _) = tokio_tungstenite::connect_async(&url).await.map_err(|e| {
        WebSocketError::Connection(ConnectionError::IOError(e))
    })?;

    verify_conn(&mut stream).await.map_err(|e| {
        WebSocketError::Connection(e)
    })?;

    log::info!("Connected to {}", url);

    Ok(stream)
}
