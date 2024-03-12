use bucface_utils::{EventDBErrorSerde, ServerResponse};
use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, StreamExt};
use std::io;
use std::sync::atomic::AtomicU64;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use surrealdb::Surreal;
use tokio::net::TcpListener;
use tokio::sync;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;

use crate::app::handle_client_message;
use crate::db;

pub async fn handle_connection<T: surrealdb::Connection>(
    mut read: ClientWsFaucet,
    write: Sender<ServerResponse>,
    db: Surreal<T>,
    id_counter: Arc<AtomicU64>,
) -> Result<(), io::Error> {
    while let Some(Ok(msg)) = read.next().await {
        match msg {
            Message::Ping(inner_msg) => {
                log::debug!("Received ping: {inner_msg:?}");
                write.send(ServerResponse::Pong(inner_msg)).unwrap();
            }
            Message::Pong(inner_msg) => {
                log::debug!("Unexpectedly received pong: {inner_msg:?}",);
            }
            Message::Close(inner_msg) => {
                log::debug!("Received close with message: {inner_msg:?}",);
                write
                    .send(ServerResponse::Close("Received close request".into()))
                    .unwrap();
                break;
            }
            Message::Text(inner_msg) => {
                log::debug!("Received text: {inner_msg}",);
            }
            Message::Binary(inner_msg) => {
                log::debug!("Received binary: {inner_msg:?}",);
                handle_binary_message(&inner_msg, &db, id_counter.clone(), &write).await
            }
            Message::Frame(inner_msg) => {
                log::debug!("Received frame: {inner_msg}",);
            }
        }
    }

    Ok(())
}

async fn start_sender(read: Receiver<ServerResponse>, sinks: Arc<sync::Mutex<Vec<ClientWsSink>>>) {
    while let Ok(res) = read.recv() {
        for (i, sink) in sinks.lock().await.iter_mut().enumerate() {
            let result_encoded = rmp_serde::encode::to_vec(&res).expect("Error encoding response");
            match sink.send(Message::Binary(result_encoded)).await {
                Ok(_) => {}
                Err(tokio_tungstenite::tungstenite::Error::AlreadyClosed) => {
                    log::info!("Removing closed sink");
                    let _ = sinks.lock().await.remove(i);
                }
                Err(e) => {
                    log::error!("Error sending message: {e:?}");
                }
            }
        }
    }
}

async fn handle_binary_message<T: surrealdb::Connection>(
    message: &[u8],
    db: &Surreal<T>,
    id_counter: Arc<AtomicU64>,
    sender_writer: &Sender<ServerResponse>,
) {
    let result = handle_client_message(message, db, id_counter).await;
    match result {
        Ok(db_events) => {
            for db_event in db_events {
                let response = ServerResponse::Event(db_event);
                sender_writer.send(response).unwrap();
            }
        }
        Err(e) => {
            log::error!("Error responding to request: {e:?}");
            let response = ServerResponse::Error(EventDBErrorSerde::from(e));
            sender_writer.send(response).unwrap();
        }
    }
}

type ClientWsSink = SplitSink<WebSocketStream<tokio::net::TcpStream>, Message>;
type ClientWsFaucet = SplitStream<WebSocketStream<tokio::net::TcpStream>>;

pub async fn start<T: surrealdb::Connection>(
    db: &mut Surreal<T>,
    addr: &str,
) -> Result<(), io::Error> {
    let socket = TcpListener::bind(addr).await?;
    db::start_db(db).await.unwrap();
    let id_counter = Arc::new(AtomicU64::new(0));
    let (tx, rx) = mpsc::channel::<ServerResponse>();
    let clients: Arc<sync::Mutex<Vec<ClientWsSink>>> = Arc::new(sync::Mutex::new(Vec::new()));

    let sender_clients = clients.clone();
    tokio::spawn(async move {
        start_sender(rx, sender_clients).await;
    });

    while let Ok((stream, addr)) = socket.accept().await {
        log::info!("Accepted connection from: {addr:?}");
        let ws_stream = tokio_tungstenite::accept_async(stream)
            .await
            .expect("Error during websocket handshake");

        let (write, read) = ws_stream.split();
        let mut clients_unlocked = clients.lock().await;
        clients_unlocked.push(write);

        let tx_clone = tx.clone();
        let db_clone = db.clone();
        let id_counter_clone = id_counter.clone();
        tokio::spawn(async move {
            handle_connection(read, tx_clone, db_clone, id_counter_clone)
                .await
                .expect("Error handling connection");
        });
    }

    Ok(())
}
