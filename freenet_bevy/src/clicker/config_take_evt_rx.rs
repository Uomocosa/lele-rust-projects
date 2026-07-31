use tokio::sync::mpsc::UnboundedReceiver;

use super::config::Config;
use crate::clicker::event::Event;

pub fn take_evt_rx(config: &Config) -> UnboundedReceiver<Event> {
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
    use crate::clicker::Config;

    #[test]
    fn test_usage() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let (_evt_tx, evt_rx) = mpsc::unbounded_channel();
        let key = freenet_stdlib::prelude::ContractKey::from_params_and_code(
            freenet_stdlib::prelude::Parameters::from(Vec::new()),
            freenet_stdlib::prelude::ContractCode::from(Vec::new()),
        );
        let cfg = Config::new(tx, evt_rx, key, 5);
        let rx = take_evt_rx(&cfg);
        drop(rx);
    }
}
