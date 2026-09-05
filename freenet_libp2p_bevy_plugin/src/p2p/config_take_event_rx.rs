use tokio::sync::mpsc::UnboundedReceiver;

use super::config::Config;
use crate::p2p;

pub fn take_event_rx<T: p2p::Message>(
    config: &Config<T>,
) -> Option<UnboundedReceiver<p2p::Event<T>>> {
    config.event_rx.lock().ok()?.take()
}

#[cfg(test)]
mod tests {
    use tokio::sync::mpsc;

    use super::take_event_rx;
    use crate::net_id::NetworkId;
    use crate::p2p;
    use derive_more::Deref;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Deref)]
    struct Dummy(u32);

    #[test]
    fn test_usage() {
        let (cmd_tx, _) = mpsc::unbounded_channel();
        let (_, event_rx) = mpsc::unbounded_channel::<p2p::Event<Dummy>>();
        let cfg = p2p::Config::new(NetworkId(1), cmd_tx, event_rx);
        assert!(take_event_rx(&cfg).is_some());
        assert!(take_event_rx(&cfg).is_none());
    }
}
