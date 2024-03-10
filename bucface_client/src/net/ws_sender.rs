use bucface_utils::ws::WsWriter;
use bucface_utils::Event;
use futures_util::SinkExt;
use serde::Serialize;
use tokio_tungstenite::tungstenite::{self, Message};

#[derive(Debug)]
pub enum SendLogError {
    EncodeError(rmp_serde::encode::Error),
    SendError(tungstenite::Error),
}

pub async fn send_log(
    log: Event,
    writer: &mut WsWriter,
) -> Result<(), SendLogError> {
    log::debug!("Sending log: {log:?}");
    let mut buf = Vec::new();
    let mut serializer = rmp_serde::Serializer::new(&mut buf);
    log.serialize(&mut serializer).map_err(SendLogError::EncodeError)?;

    let message = Message::Binary(buf);

    writer.send(message).await.map_err(SendLogError::SendError)
}
