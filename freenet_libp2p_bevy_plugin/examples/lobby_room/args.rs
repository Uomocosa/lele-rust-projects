use std::str::FromStr;

use clap::{Parser, ValueEnum};
use freenet_libp2p_bevy_plugin::discovery::id::RoomName;
use freenet_libp2p_bevy_plugin::p2p::TransportMode;

#[derive(Debug, Clone)]
pub enum Action {
    CreateRoom(RoomName),
    JoinRoom(RoomName),
}

impl FromStr for Action {
    type Err = String;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let (kind, room) = raw
            .split_once(':')
            .ok_or_else(|| format!("expected create:<room> or join:<room>, got {raw}"))?;
        if room.is_empty() {
            return Err("room name is empty".to_string());
        }
        let room = RoomName(room.to_string());
        match kind {
            "create" => Ok(Self::CreateRoom(room)),
            "join" => Ok(Self::JoinRoom(room)),
            other => Err(format!("unknown action {other}, expected create or join")),
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum TransportArg {
    Tcp,
    Quic,
    Both,
}

#[derive(Debug, Parser)]
#[command(name = "lobby_room")]
pub struct Args {
    #[arg(long)]
    pub username: String,
    #[arg(long)]
    pub action: Option<Action>,
    #[arg(long, value_enum, default_value = "both")]
    pub transport: TransportArg,
}

#[must_use]
pub const fn transport_mode(arg: TransportArg) -> TransportMode {
    match arg {
        TransportArg::Tcp => TransportMode::Tcp,
        TransportArg::Quic => TransportMode::Quic,
        TransportArg::Both => TransportMode::Both,
    }
}
