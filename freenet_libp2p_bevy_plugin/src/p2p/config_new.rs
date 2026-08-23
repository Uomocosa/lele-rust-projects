use std::sync::Mutex;

use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use super::config::Config;
use crate::p2p;

pub fn new<T: p2p::Message>(
    cmd_tx: UnboundedSender<p2p::Command<T>>,
    event_rx: UnboundedReceiver<p2p::Event<T>>,
) -> Config<T> {
    Config {
        cmd_tx,
        event_rx: Mutex::new(Some(event_rx)),
    }
}

#[cfg(test)]
mod tests {
    use derive_more::Deref;
    use serde::{Deserialize, Serialize};

    use tokio::sync::mpsc;

    use super::new;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Deref)]
    struct Dummy(u32);

    #[test]
    fn test_usage() {
        let (cmd_tx, _cmd_rx) = mpsc::unbounded_channel();
        let (_event_tx, event_rx) = mpsc::unbounded_channel();
        let config = new::<Dummy>(cmd_tx, event_rx);
        assert!(config.event_rx.lock().ok().is_some());
    }
}
