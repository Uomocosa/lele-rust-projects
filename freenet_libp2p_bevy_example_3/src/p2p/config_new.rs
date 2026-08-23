use std::sync::Mutex;

use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use super::config::Config;
use crate::p2p;

pub fn new(
    cmd_tx: UnboundedSender<p2p::Command>,
    event_rx: UnboundedReceiver<p2p::Event>,
) -> Config {
    Config {
        cmd_tx,
        event_rx: Mutex::new(Some(event_rx)),
    }
}

#[cfg(test)]
mod tests {
    use tokio::sync::mpsc;

    use super::new;

    #[test]
    fn test_usage() {
        let (cmd_tx, _cmd_rx) = mpsc::unbounded_channel();
        let (_event_tx, event_rx) = mpsc::unbounded_channel();
        let config = new(cmd_tx, event_rx);
        assert!(config.event_rx.lock().ok().is_some());
    }
}
