use std::io;

use bucface_utils::ws::WsReader;
use bucface_utils::Event;
use futures_util::StreamExt;
use rmp_serde::decode;
use tokio::sync::mpsc::{self, Sender};
use tokio_tungstenite::tungstenite::Message;

pub async fn handle_connection(
    read: &mut WsReader,
    tx: &Sender<Event>,
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
    DecodeError(decode::Error),
    SendError(mpsc::error::SendError<Event>),
}

fn receive_event(tx: Sender<Event>, data: Vec<u8>) -> Result<(), ReceiveEventError> {
    let events = rmp_serde::from_slice::<Event>(&data).map_err(ReceiveEventError::DecodeError)?;
    tx.blocking_send(events)
        .map_err(ReceiveEventError::SendError)?;

    Ok(())
}
