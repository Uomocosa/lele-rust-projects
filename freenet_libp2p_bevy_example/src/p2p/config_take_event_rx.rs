use tokio::sync::mpsc::UnboundedReceiver;

use super::config::Config;
use crate::p2p;

pub fn take_event_rx(config: &Config) -> UnboundedReceiver<p2p::Event> {
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
    use tokio::sync::mpsc;

    use super::take_event_rx;
    use crate::p2p;

    #[test]
    fn test_usage() {
        let (cmd_tx, _cmd_rx) = mpsc::unbounded_channel();
        let (_event_tx, event_rx) = mpsc::unbounded_channel();
        let config = p2p::Config::new(cmd_tx, event_rx);
        let rx = take_event_rx(&config);
        drop(rx);
    }
}
