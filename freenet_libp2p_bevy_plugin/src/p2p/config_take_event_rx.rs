use tokio::sync::mpsc::UnboundedReceiver;

use super::config::Config;
use crate::p2p;

pub fn take_event_rx<T: p2p::Message>(config: &Config<T>) -> UnboundedReceiver<p2p::Event<T>> {
    config
        .event_rx
        .lock()
        .unwrap()
        .take()
        .expect("event_rx already taken")
}

#[cfg(test)]
mod tests {
    use tokio::sync::mpsc;

    use crate::net_id::NetworkId;
    use crate::p2p;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct Dummy(u32);

    #[test]
    fn test_usage() {
        let (cmd_tx, _) = mpsc::unbounded_channel();
        let (_, event_rx) = mpsc::unbounded_channel::<p2p::Event<Dummy>>();
        let cfg = p2p::Config::new(NetworkId(1), cmd_tx, event_rx);
        let rx = take_event_rx(&cfg);
        assert!(rx.is_empty());
    }
}
