use bevy::prelude::Resource;
use tokio::sync::watch::{self, Receiver, Sender};

use crate::p2p;
use p2p::Ready;

#[derive(Resource)]
pub struct Signals {
    pub ready_tx: Sender<Option<Ready>>,
    pub ready_rx: Receiver<Option<Ready>>,
    pub observed_tx: Sender<Option<Vec<String>>>,
    pub observed_rx: Receiver<Option<Vec<String>>>,
}

#[rustfmt::skip]
impl Signals {
    #[must_use]
    pub fn subscribe(&self) -> (Receiver<Option<Ready>>, Receiver<Option<Vec<String>>>) {
        (self.ready_rx.clone(), self.observed_rx.clone())
    }
}

impl Signals {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for Signals {
    fn default() -> Self {
        let (ready_tx, ready_rx) = watch::channel(None);
        let (observed_tx, observed_rx) = watch::channel(None);
        Self {
            ready_tx,
            ready_rx,
            observed_tx,
            observed_rx,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Signals;
    use crate::p2p;

    #[test]
    fn test_usage() {
        let signals = Signals::default();
        let (ready_rx, observed_rx) = signals.subscribe();
        signals.ready_tx.send_replace(Some(p2p::Ready {
            peer_id: "p".to_string(),
            addrs: vec![],
        }));
        signals
            .observed_tx
            .send_replace(Some(vec!["/ip4/1.2.3.4/tcp/1".to_string()]));
        assert_eq!(
            ready_rx.borrow().as_ref().map(|r| r.peer_id.as_str()),
            Some("p")
        );
        assert!(observed_rx.borrow().is_some());
    }
}
