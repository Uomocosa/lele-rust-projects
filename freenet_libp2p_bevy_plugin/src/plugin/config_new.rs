use std::sync::Mutex;

use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use super::config::Config;
use crate::net_id;
use crate::p2p;

pub fn new<T: p2p::Message>(
    own_id: net_id::NetworkId,
    cmd_tx: UnboundedSender<p2p::Command<T>>,
    event_rx: UnboundedReceiver<p2p::Event<T>>,
) -> Config<T> {
    Config {
        own_id,
        cmd_tx,
        event_rx: Mutex::new(Some(event_rx)),
    }
}

#[cfg(test)]
mod tests {
    use tokio::sync::mpsc;

    use super::new;
    use crate::net_id::NetworkId;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct Dummy(u32);

    #[test]
    fn test_usage() {
        let (cmd_tx, _) = mpsc::unbounded_channel();
        let (_, event_rx) = mpsc::unbounded_channel();
        let cfg = new::<Dummy>(NetworkId(1), cmd_tx, event_rx);
        assert_eq!(*cfg.own_id, 1);
    }
}
