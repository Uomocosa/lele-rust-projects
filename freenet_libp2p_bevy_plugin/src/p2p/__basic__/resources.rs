use std::sync::Mutex;

use bevy::prelude::Resource;
use derive_more::Deref;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use super::enums::{Command, Event, NetCommand};
use crate::p2p;

#[derive(Resource)]
pub struct Bridge<T: p2p::Message> {
    pub cmd_tx: UnboundedSender<Command<T>>,
    pub event_rx: Mutex<Option<UnboundedReceiver<Event<T>>>>,
}

#[derive(Resource, Deref)]
pub struct NetBridge(pub UnboundedSender<NetCommand>);

#[cfg(test)]
mod tests {
    use super::{Bridge, NetBridge};
    use crate::p2p;

    #[test]
    fn test_usage() {
        let (cmd_tx, _cmd_rx) = tokio::sync::mpsc::unbounded_channel();
        let (_event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel::<p2p::Event<()>>();
        let bridge = Bridge {
            cmd_tx,
            event_rx: std::sync::Mutex::new(Some(event_rx)),
        };
        assert!(!bridge.cmd_tx.is_closed());

        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel::<p2p::NetCommand>();
        let net = NetBridge(tx);
        assert!(!net.is_closed());
    }
}
