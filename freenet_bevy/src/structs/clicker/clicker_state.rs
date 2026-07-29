use std::sync::Mutex;

use bevy::prelude::Resource;
use freenet_stdlib::prelude::ContractKey;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::structs::clicker::clicker_command::ClickerCommand;
use crate::structs::clicker::clicker_event::ClickerEvent;

#[derive(Resource)]
pub struct ClickerState {
    pub event_rx: Mutex<UnboundedReceiver<ClickerEvent>>,
    pub cmd_tx: UnboundedSender<ClickerCommand>,
    pub contract_key: ContractKey,
    pub count: u64,
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::ClickerState;
    use tokio::sync::mpsc;

    #[test]
    fn test_usage() {
        let (cmd_tx, _cmd_rx) = mpsc::unbounded_channel();
        let (_evt_tx, evt_rx) = mpsc::unbounded_channel();
        let key = freenet_stdlib::prelude::ContractKey::from_params_and_code(
            freenet_stdlib::prelude::Parameters::from(Vec::new()),
            freenet_stdlib::prelude::ContractCode::from(Vec::new()),
        );
        let _state = ClickerState {
            event_rx: Mutex::new(evt_rx),
            cmd_tx,
            contract_key: key,
            count: 42,
        };
    }
}
