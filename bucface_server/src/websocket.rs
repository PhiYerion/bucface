use futures::{SinkExt, StreamExt};
use std::io;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::Message;

pub async fn handle_connection(stream: TcpStream, socket: SocketAddr) -> Result<(), io::Error> {
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
                write.send(Message::Binary(inner_msg)).await.unwrap();
            }
            Message::Frame(inner_msg) => {
                log::debug!("Received frame from {socket}: {inner_msg}",);
                write.send(Message::Frame(inner_msg)).await.unwrap();
            }
        }
    }

    Ok(())
}

pub async fn start() -> Result<(), io::Error> {
    let socket = TcpListener::bind("0.0.0.0:8080").await?;

    while let Ok((stream, addr)) = socket.accept().await {
        tokio::spawn(handle_connection(stream, addr));
    }

    Ok(())
}
