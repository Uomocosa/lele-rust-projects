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
        let key = freenet_stdlib::prelude::ContractKey::from_params_and_code(
            freenet_stdlib::prelude::Parameters::from(Vec::new()),
            freenet_stdlib::prelude::ContractCode::from(Vec::new()),
        );
        let cfg = clicker::Config::new(tx, evt_rx, key, 5);
        let rx = take_evt_rx(&cfg);
        drop(rx);
    }
}
