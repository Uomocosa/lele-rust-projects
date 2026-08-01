use std::sync::Mutex;

use freenet_stdlib::prelude::ContractKey;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::clicker;

pub fn new(
    cmd_tx: UnboundedSender<clicker::Command>,
    evt_rx: UnboundedReceiver<clicker::Event>,
    contract_key: ContractKey,
    initial_count: u64,
) -> clicker::Config {
    clicker::Config {
        cmd_tx,
        evt_rx: Mutex::new(Some(evt_rx)),
        contract_key,
        initial_count,
    }
}

#[cfg(test)]
mod tests {
    use tokio::sync::mpsc;

    use super::new;

    #[test]
    fn test_usage() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let (_evt_tx, evt_rx) = mpsc::unbounded_channel();
        let key = freenet_stdlib::prelude::ContractKey::from_params_and_code(
            freenet_stdlib::prelude::Parameters::from(Vec::new()),
            freenet_stdlib::prelude::ContractCode::from(Vec::new()),
        );
        let cfg = new(tx, evt_rx, key, 42);
        assert_eq!(cfg.initial_count, 42);
    }
}
