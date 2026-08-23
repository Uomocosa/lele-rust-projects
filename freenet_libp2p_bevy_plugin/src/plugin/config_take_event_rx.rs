use tokio::sync::mpsc::UnboundedReceiver;

use super::config::Config;
use crate::p2p;

pub fn take_event_rx<T: p2p::Message>(config: &Config<T>) -> UnboundedReceiver<p2p::Event<T>> {
    config
        .event_rx
        .lock()
        .ok()
        .and_then(|mut slot| slot.take())
        .unwrap_or_else(|| {
            let (_, rx) = tokio::sync::mpsc::unbounded_channel();
            rx
        })
}

#[cfg(test)]
mod tests {
    use derive_more::Deref;
    use serde::{Deserialize, Serialize};

    use tokio::sync::mpsc;

    use super::take_event_rx;
    use crate::net_id;
    use crate::plugin;
    use crate::roster;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Deref)]
    struct Dummy(u32);

    #[test]
    fn test_usage() {
        let (cmd_tx, _cmd_rx) = mpsc::unbounded_channel();
        let (_event_tx, event_rx) = mpsc::unbounded_channel();
        let (_roster_tx, roster_rx) = mpsc::unbounded_channel::<roster::Event>();
        let config =
            plugin::Config::<Dummy>::new(net_id::NetworkId(7), cmd_tx, event_rx, roster_rx);
        let rx = take_event_rx(&config);
        drop(rx);
    }
}
