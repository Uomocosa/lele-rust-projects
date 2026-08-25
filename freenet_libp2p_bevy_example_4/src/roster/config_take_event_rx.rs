use tokio::sync::mpsc::UnboundedReceiver;

use crate::roster;

pub fn take_event_rx(config: &roster::Config) -> UnboundedReceiver<roster::Event> {
    config
        .lock()
        .unwrap()
        .take()
        .expect("event_rx already taken")
}

#[cfg(test)]
mod tests {
    use tokio::sync::mpsc;

    use super::take_event_rx;
    use crate::roster;

    #[test]
    fn test_usage() {
        let (_tx, rx) = mpsc::unbounded_channel();
        let config = roster::Config::new(rx);
        let rx = take_event_rx(&config);
        drop(rx);
    }
}
