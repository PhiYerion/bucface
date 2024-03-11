use bucface_utils::EventDBResponse;
use futures::{SinkExt, StreamExt};
use std::io;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicI64, AtomicUsize, Ordering};
use std::sync::Arc;
use surrealdb::engine::local::Mem;
use surrealdb::{Connection, Surreal};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::Message;

use crate::app::handle_new_event;
use crate::db;

pub async fn handle_connection<T: surrealdb::Connection>(
    stream: TcpStream,
    socket: SocketAddr,
    db: Surreal<T>,
    id_counter: Arc<AtomicI64>,
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
                let id = id_counter.fetch_add(1, Ordering::Relaxed);
                let result = handle_new_event(&inner_msg, db.clone(), id).await;
                let response = match result {
                    Ok(event) => {
                        log::debug!("Inserted event: {event:?}");
                        EventDBResponse {
                            id,
                            inner: Ok(event),
                        }
                    }
                    Err(e) => {
                        log::error!("Error inserting event: {e:?}");
                        EventDBResponse::from_err(id, e)
                    }
                };

                let result_encoded = rmp_serde::encode::to_vec(&response).unwrap();

                write.send(Message::Binary(result_encoded)).await.unwrap();
            }
            Message::Frame(inner_msg) => {
                log::debug!("Received frame from {socket}: {inner_msg}",);
                write.send(Message::Frame(inner_msg)).await.unwrap();
            }
        }
    }

    Ok(())
}

pub async fn start<T: surrealdb::Connection>(
    db: &mut Surreal<T>,
    addr: &str,
) -> Result<(), io::Error> {
    let socket = TcpListener::bind(addr).await?;
    db::start_db(db).await.unwrap();
    let id_counter = Arc::new(AtomicI64::new(0));

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
