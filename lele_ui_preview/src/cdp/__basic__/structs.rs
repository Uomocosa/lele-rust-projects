use std::net::TcpStream;
use std::process::Child;

use tungstenite::WebSocket;
use tungstenite::stream::MaybeTlsStream;

pub struct Browser {
    pub child: Child,
    pub port: u16,
}

pub struct Session {
    pub socket: WebSocket<MaybeTlsStream<TcpStream>>,
    pub next_id: u64,
}
