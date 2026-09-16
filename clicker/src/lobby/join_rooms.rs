use freenet_libp2p_bevy_plugin::roster;

use crate::clicker;
use crate::lobby;

pub struct JoinRooms<'a> {
    pub active: &'a mut clicker::ActiveLobby,
    pub selected: &'a mut lobby::SelectedRoom,
    pub roster_lobby: &'a mut roster::Lobby,
    pub pending: &'a mut lobby::JoinPending,
    pub gate: &'a mut lobby::JoinGate,
    pub clock: &'a mut lobby::JoinClock,
}
