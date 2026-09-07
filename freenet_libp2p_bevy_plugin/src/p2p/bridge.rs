use std::sync::Mutex;

use bevy::prelude::Resource;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::p2p;

#[derive(Resource)]
pub struct Bridge<T: p2p::Message> {
    pub cmd_tx: UnboundedSender<p2p::Command<T>>,
    pub event_rx: Mutex<Option<UnboundedReceiver<p2p::Event<T>>>>,
}

#[cfg(test)]
mod tests {
    use super::Bridge;
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
    }
}
