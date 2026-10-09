use crate::discovery;
use crate::p2p;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Input {
    Command(discovery::Command),
    Net(p2p::NetEvent),
    Lobby(discovery::Lobby),
    Tick,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Output {
    Event(discovery::Event),
    Net(p2p::NetCommand),
}
