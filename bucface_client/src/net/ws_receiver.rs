use std::io;

use bucface_utils::ws::WsFaucet;
use bucface_utils::ServerResponse;
use egui::Context;
use futures_util::StreamExt;
use rmp_serde::decode;
use tokio::sync::mpsc::{self, Sender};
use tokio_tungstenite::tungstenite::Message;

pub async fn start_receiver(
    read: &mut WsFaucet,
    tx: &Sender<ServerResponse>,
    egui_ctx: Context,
) -> Result<(), io::Error> {
    while let Some(res) = read.next().await {
        match res {
            Ok(Message::Binary(data)) => {
                log::debug!("Received binary");
                if let Err(e) = receive_event(tx.clone(), data).await {
                    log::error!("Error receiving event: {e:?}");
                }
                egui_ctx.request_repaint();
                log::debug!("Repainting");
            }
            Ok(t) => {
                log::debug!("Received non-binary message: {t}");
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
    Decode(decode::Error),
    Send(mpsc::error::SendError<ServerResponse>),
}

async fn receive_event(
    tx: Sender<ServerResponse>,
    data: Vec<u8>,
) -> Result<(), ReceiveEventError> {
    let events =
        rmp_serde::from_slice::<ServerResponse>(&data).map_err(ReceiveEventError::Decode)?;

    tx.send(events).await.map_err(ReceiveEventError::Send)?;

    Ok(())
}
