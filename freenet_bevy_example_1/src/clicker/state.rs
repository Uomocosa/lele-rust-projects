use std::sync::Mutex;

use bevy::prelude::Resource;
use freenet_stdlib::prelude::ContractKey;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::clicker;

#[derive(Resource)]
pub struct State {
    pub event_rx: Mutex<UnboundedReceiver<clicker::Event>>,
    pub cmd_tx: UnboundedSender<clicker::Command>,
    pub contract_key: ContractKey,
    pub count: u64,
    pub connected: bool,
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::State;
    use tokio::sync::mpsc;

    #[test]
    fn test_usage() {
        let (cmd_tx, _cmd_rx) = mpsc::unbounded_channel();
        let (_evt_tx, evt_rx) = mpsc::unbounded_channel();
        let key = freenet_stdlib::prelude::ContractKey::from_params_and_code(
            freenet_stdlib::prelude::Parameters::from(Vec::new()),
            freenet_stdlib::prelude::ContractCode::from(Vec::new()),
        );
        let _state = State {
            event_rx: Mutex::new(evt_rx),
            cmd_tx,
            contract_key: key,
            count: 42,
            connected: false,
        };
    }
}
