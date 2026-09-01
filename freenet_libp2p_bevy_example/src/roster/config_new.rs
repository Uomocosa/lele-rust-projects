use std::sync::Mutex;

use tokio::sync::mpsc::UnboundedReceiver;

use crate::roster;

pub fn new(event_rx: UnboundedReceiver<roster::Event>) -> roster::Config {
    roster::Config(Mutex::new(Some(event_rx)))
}

#[cfg(test)]
mod tests {
    use tokio::sync::mpsc;

    use super::new;

    #[test]
    fn test_usage() {
        let (_tx, rx) = mpsc::unbounded_channel();
        let config = new(rx);
        assert!(config.lock().unwrap().is_some());
    }
}
