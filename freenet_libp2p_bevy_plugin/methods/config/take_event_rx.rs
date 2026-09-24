use tokio::sync::mpsc::UnboundedReceiver;

use crate::p2p;

pub fn take_event_rx<T: p2p::Message>(
    config: &p2p::Config<T>,
) -> Option<UnboundedReceiver<p2p::Event<T>>> {
    config.event_rx.lock().ok()?.take()
}

#[cfg(test)]
mod tests {
    use derive_more::Deref;
    use serde::{Deserialize, Serialize};
    use tokio::sync::mpsc;

    use super::take_event_rx;
    use crate::net_id::NetworkId;
    use crate::p2p;

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
