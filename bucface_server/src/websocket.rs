use bucface_utils::{EventDBError, EventDBResponse};
use futures::stream::SplitSink;
use futures::{SinkExt, StreamExt};
use std::io;
use std::net::SocketAddr;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use surrealdb::Surreal;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;

use crate::app::handle_client_message;
use crate::db;

pub async fn handle_connection<T: surrealdb::Connection>(
    stream: TcpStream,
    socket: SocketAddr,
    db: Surreal<T>,
    id_counter: Arc<AtomicU64>,
) -> Result<(), io::Error> {
    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Error during websocket handshake");

    let (mut write, mut read) = ws_stream.split();
    while let Some(Ok(msg)) = read.next().await {
        match msg {
            Message::Ping(inner_msg) => {
                log::debug!("Received ping from {socket}: {inner_msg:?}");
                write.send(Message::Pong(inner_msg)).await.unwrap();
            }
            Message::Pong(inner_msg) => {
                log::debug!("Unexpectedly received pong from {socket}: {inner_msg:?}",);
            }
            Message::Close(inner_msg) => {
                log::debug!("Received close from {socket} with message: {inner_msg:?}",);
                write.send(Message::Close(inner_msg)).await.unwrap();
                break;
            }
            Message::Text(inner_msg) => {
                log::debug!("Received text from {socket}: {inner_msg}",);
                write.send(Message::Text(inner_msg)).await.unwrap();
            }
            Message::Binary(inner_msg) => {
                log::debug!("Received binary from {socket}: {inner_msg:?}",);
                handle_binary_message(&inner_msg, &db, id_counter.clone(), &mut write)
                    .await
                    .expect("Error handling binary message");
            }
            Message::Frame(inner_msg) => {
                log::debug!("Received frame from {socket}: {inner_msg}",);
                write.send(Message::Frame(inner_msg)).await.unwrap();
            }
        }
    }

    Ok(())
}

async fn handle_binary_message<T: surrealdb::Connection>(
    message: &[u8],
    db: &Surreal<T>,
    id_counter: Arc<AtomicU64>,
    write: &mut SplitSink<WebSocketStream<TcpStream>, Message>,
) -> Result<(), EventDBError> {
    let result = handle_client_message(message, db, id_counter).await;
    match result {
        Ok(db_events) => {
            for db_event in db_events {
                log::debug!("Inserted event: {db_event:?}");
                let response = EventDBResponse::from(db_event);
                let result_encoded =
                    rmp_serde::encode::to_vec(&response).map_err(EventDBError::RmpEncode)?;
                write.send(Message::Binary(result_encoded)).await.unwrap();
            }
        }
        Err((e, id)) => {
            log::error!("Error inserting event: {e:?}");
            let response = EventDBResponse::from_err(id, e);
            let result_encoded =
                rmp_serde::encode::to_vec(&response).map_err(EventDBError::RmpEncode)?;
            write.send(Message::Binary(result_encoded)).await.unwrap();
        }
    };

    Ok(())
}

pub async fn start<T: surrealdb::Connection>(
    db: &mut Surreal<T>,
    addr: &str,
) -> Result<(), io::Error> {
    let socket = TcpListener::bind(addr).await?;
    db::start_db(db).await.unwrap();
    let id_counter = Arc::new(AtomicU64::new(0));

    while let Ok((stream, addr)) = socket.accept().await {
        tokio::spawn(handle_connection(
            stream,
            addr,
            db.clone(),
            id_counter.clone(),
        ));
    }

    Ok(())
}
