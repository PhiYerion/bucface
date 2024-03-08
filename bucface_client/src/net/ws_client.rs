use bucface_utils::Events;
use futures_util::stream::SplitSink;
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{connect_async, MaybeTlsStream};
use tokio_tungstenite::{tungstenite, WebSocketStream};

use super::ws_receiver::handle_connection;

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

#[derive(Debug)]
pub enum VerifyError {
    NoResponse,
    InvalidResponse,
    IOError(tungstenite::Error),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WebSocketStatus {
    Success,
    ConnectionLost(String),
    SoftError(String),
    HardError(String),
}

pub async fn verify_conn(stream: &mut WsStream) -> Result<WebSocketStatus, VerifyError> {
    log::debug!("Verifying WebSocket connection");

    const ECHO: &[u8] = b"echo\n";
    let (mut writer, mut reader) = stream.split();

    log::trace!("Sending ping");
    writer
        .send(tungstenite::Message::Ping(ECHO.to_vec()))
        .await
        .map_err(VerifyError::IOError)?;

    while let Some(msg) = reader.next().await {
        log::trace!("Received message: {:?}", msg);
        let msg = msg.map_err(VerifyError::IOError)?;
        if msg.is_pong() {
            let data = msg.into_data();
            if data != ECHO {
                log::warn!("Invalid pong response, treating as non-fatal: {data:?}");
                return Ok(WebSocketStatus::SoftError(
                    "Invalid pong response: ".to_string() + &String::from_utf8_lossy(&data),
                ));
            }

            log::debug!("WebSocket connection verified");
            return Ok(WebSocketStatus::Success);
        }
    }

    log::warn!("No response to ping");
    Err(VerifyError::NoResponse)
}

async fn connect(dest: &str) -> Result<WsStream, WebSocketStatus> {
    log::info!("Connecting to {}", dest);
    let (mut stream, _) = tokio_tungstenite::connect_async(dest).await.map_err(|e| {
        log::error!("Error connecting to {}: {:?}", dest, e);
        WebSocketStatus::HardError(format!("Error connecting to {}: {:?}", dest, e))
    })?;

    verify_conn(&mut stream).await.map_err(|e| {
        log::error!("Error verifying connection: {:?}", e);
        WebSocketStatus::HardError("Error verifying connection: {e:?}".to_string())
    })?;
    log::trace!("Connected to {}", dest);

    Ok(stream)
}

#[derive(Debug)]
pub struct WsClient {
    pub receiver: Receiver,
    pub write: SplitSink<WsStream, Message>,
}

#[derive(Debug)]
pub struct Receiver {
    pub receiver: tokio::task::JoinHandle<()>,
    pub rx: tokio::sync::mpsc::Receiver<Events>,
}

pub async fn start(dest: &str) -> WsClient {
    let (stream, _) = connect_async(dest)
        .await
        .map_err(|e| {
            log::error!("Error connecting to {}: {:?}", dest, e);
            WebSocketStatus::HardError(format!("Error connecting to {}: {:?}", dest, e))
        })
        .unwrap();

    let (write, mut read) = stream.split();

    let (receiver_tx, receiver_rx) = tokio::sync::mpsc::channel::<Events>(1);
    let receiver = tokio::spawn(async move {
        handle_connection(&mut read, &receiver_tx).await.unwrap();
    });

    WsClient {
        receiver: Receiver {
            receiver,
            rx: receiver_rx,
        },
        write,
    }
}
