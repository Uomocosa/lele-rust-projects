use tokio::sync::mpsc::UnboundedReceiver;

use crate::clicker;

pub fn take_evt_rx(config: &clicker::Config) -> UnboundedReceiver<clicker::Event> {
    config
        .evt_rx
        .lock()
        .unwrap()
        .take()
        .expect("evt_rx already taken")
}

#[cfg(test)]
mod tests {
    use tokio::sync::mpsc;

    use super::take_evt_rx;
    use crate::clicker;

    #[test]
    fn test_usage() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let (_evt_tx, evt_rx) = mpsc::unbounded_channel();
        let cfg = clicker::Config::new(tx, evt_rx);
        let rx = take_evt_rx(&cfg);
        drop(rx);
    }
}
