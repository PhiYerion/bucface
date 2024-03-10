mod events;
mod websocket;
mod app;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    websocket::start().await
}
