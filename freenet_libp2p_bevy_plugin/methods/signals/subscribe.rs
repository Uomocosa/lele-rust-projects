use tokio::sync::watch::Receiver;

use crate::p2p;

#[must_use]
pub fn subscribe(
    signals: &p2p::Signals,
) -> (Receiver<Option<p2p::Ready>>, Receiver<Option<Vec<String>>>) {
    (signals.ready_rx.clone(), signals.observed_rx.clone())
}

#[cfg(test)]
mod tests {
    use super::subscribe;
    use crate::p2p;

    #[test]
    fn test_usage() {
        let signals = p2p::Signals::default();
        let (ready_rx, observed_rx) = subscribe(&signals);
        signals.ready_tx.send_replace(Some(p2p::Ready {
            peer_id: "p".to_string(),
            addrs: vec![],
        }));
        assert!(ready_rx.borrow().is_some());
        assert!(observed_rx.borrow().is_none());
    }
}
