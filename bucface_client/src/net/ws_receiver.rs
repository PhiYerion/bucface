use std::io;

use bucface_utils::Events;
use futures_util::stream::SplitStream;
use futures_util::StreamExt;
use tokio::net::TcpStream;
use tokio::sync::mpsc::Sender;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

pub async fn handle_connection(
    read: &mut SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    tx: &Sender<Events>,
) -> Result<(), io::Error> {
    while let Some(res) = read.next().await {
        match res {
            Ok(Message::Binary(data)) => {
                log::debug!("Received binary");
                if let Err(e) = receive_event(tx.clone(), data) {
                    log::error!("Error receiving event: {e:?}");
                }
            }
            Ok(_) => {
                log::debug!("Received non-binary message");
            }
            Err(e) => {
                log::error!("Error receiving message: {e:?}");
            }
        }
    }

    Ok(())
}

#[derive(Debug)]
enum ReceiveEventError {
    DecodeError(rmp_serde::decode::Error),
    SendError(tokio::sync::mpsc::error::SendError<Events>),
}
fn receive_event(tx: Sender<Events>, data: Vec<u8>) -> Result<(), ReceiveEventError> {
    let events = rmp_serde::from_slice::<Events>(&data).map_err(ReceiveEventError::DecodeError)?;
    tx.blocking_send(events)
        .map_err(ReceiveEventError::SendError)?;

    Ok(())
}
