use bucface_utils::ws::WsSink;
use bucface_utils::Event;
use futures_util::SinkExt;
use serde::Serialize;
use tokio::sync::mpsc::Receiver;
use tokio_tungstenite::tungstenite::{self, Message};

#[derive(Debug)]
pub enum SendLogError {
    EncodeError(rmp_serde::encode::Error),
    SendError(tungstenite::Error),
}

/// Start the sender thread, which sends logs to the server
///
/// # Arguments
/// * `writer` - A [websocket](tokio_tungstenite::WebSocketStream) [writer](futures_util::stream::SplitSink)
/// connected to the server
/// * `sender_sink` - A [channel](Receiver) for the thread to receive [Event]s to send to the
/// [server](bucface_server)
pub async fn start_sender(writer: &mut WsSink, sender_sink: &mut Receiver<Event>) {
    while let Some(log) = sender_sink.recv().await {
        let counter = 0;
        log::trace!("Sending log: {log:?}");
        while let Err(e) = send_log(log.clone(), writer).await {
            log::warn!("Error sending log: {e:?}");
            if counter > 10 {
                log::error!("Failed to send log {log:?} after 10 retries. Aborting.");
                break;
            }
        }
        log::trace!("Sent log: {log:?}");
    }
}

pub async fn send_log(log: Event, writer: &mut WsSink) -> Result<(), SendLogError> {
    log::debug!("Sending log: {log:?}");
    let mut buf = Vec::new();
    let mut serializer = rmp_serde::Serializer::new(&mut buf);
    log.serialize(&mut serializer)
        .map_err(SendLogError::EncodeError)?;

    let message = Message::Binary(buf);

    writer.send(message).await.map_err(SendLogError::SendError)
}
