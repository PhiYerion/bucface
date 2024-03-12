use bucface_utils::ws::WsSink;
use bucface_utils::ClientMessage;
use futures_util::SinkExt;
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
pub async fn start_sender(writer: &mut WsSink, sender_sink: &mut Receiver<ClientMessage>) {
    while let Some(message) = sender_sink.recv().await {
        log::trace!("Sending message: {message:?}");
        let counter = 0;

        while let Err(e) = send_message(message.clone(), writer).await {
            log::warn!("Error sending message: {e:?}, trying again");
            if counter > 10 {
                log::error!("Failed to send message {message:?} after 10 retries. Aborting.");
                break;
            }
        }
        log::trace!("Sent message: {message:?}");
    }
}

pub async fn send_message(message: ClientMessage, writer: &mut WsSink) -> Result<(), SendLogError> {
    let encoded_message = rmp_serde::to_vec(&message).map_err(SendLogError::EncodeError)?;
    let message = Message::Binary(encoded_message);
    writer.send(message).await.map_err(SendLogError::SendError)
}
