use surrealdb::engine::local::Mem;
use surrealdb::Surreal;

mod app;
mod db;
mod websocket;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    let mut db = Surreal::new::<Mem>(()).await.unwrap();
    websocket::start(&mut db, "0.0.0.0:8080").await
}
