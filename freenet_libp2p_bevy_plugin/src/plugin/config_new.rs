use std::sync::Mutex;

use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use super::config::Config;
use crate::net_id;
use crate::p2p;
use crate::roster;

pub fn new<T: p2p::Message>(
    own_id: net_id::NetworkId,
    cmd_tx: UnboundedSender<p2p::Command<T>>,
    event_rx: UnboundedReceiver<p2p::Event<T>>,
    roster_rx: UnboundedReceiver<roster::Event>,
) -> Config<T> {
    Config {
        own_id,
        cmd_tx,
        event_rx: Mutex::new(Some(event_rx)),
        roster_rx: Mutex::new(Some(roster_rx)),
    }
}

#[cfg(test)]
mod tests {
    use derive_more::Deref;
    use serde::{Deserialize, Serialize};

    use tokio::sync::mpsc;

    use super::new;
    use crate::net_id;
    use crate::roster;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Deref)]
    struct Dummy(u32);

    #[test]
    fn test_usage() {
        let (cmd_tx, _cmd_rx) = mpsc::unbounded_channel();
        let (_event_tx, event_rx) = mpsc::unbounded_channel();
        let (_roster_tx, roster_rx) = mpsc::unbounded_channel::<roster::Event>();
        let config = new::<Dummy>(net_id::NetworkId(7), cmd_tx, event_rx, roster_rx);
        assert_eq!(config.own_id, net_id::NetworkId(7));
        assert!(config.event_rx.lock().ok().is_some());
        assert!(config.roster_rx.lock().ok().is_some());
    }
}
