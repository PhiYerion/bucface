use futures_util::stream::SplitSink;
use futures_util::stream::SplitStream;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::MaybeTlsStream;
use tokio_tungstenite::WebSocketStream;

pub type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;
pub type WsReader = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;
pub type WsWriter = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;
