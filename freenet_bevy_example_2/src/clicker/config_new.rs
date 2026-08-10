use std::sync::Mutex;

use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::clicker;

pub fn new(
    cmd_tx: UnboundedSender<clicker::Command>,
    evt_rx: UnboundedReceiver<clicker::Event>,
) -> clicker::Config {
    clicker::Config {
        cmd_tx,
        evt_rx: Mutex::new(Some(evt_rx)),
    }
}

#[cfg(test)]
mod tests {
    use tokio::sync::mpsc;

    use super::new;

    #[test]
    fn test_usage() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let (_evt_tx, evt_rx) = mpsc::unbounded_channel();
        let cfg = new(tx, evt_rx);
        assert!(cfg.evt_rx.lock().unwrap().is_some());
    }
}
