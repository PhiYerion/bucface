mod app;
mod websocket;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    websocket::start().await
}
