use std::collections::VecDeque;
use std::sync::Mutex;

use bevy::prelude::Resource;
use freenet_stdlib::prelude::ContractKey;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::clicker;

pub const LOG_CAPACITY: usize = 5;

#[derive(Resource)]
pub struct State {
    pub event_rx: Mutex<UnboundedReceiver<clicker::Event>>,
    pub cmd_tx: UnboundedSender<clicker::Command>,
    pub contract_key: Option<ContractKey>,
    pub count: u64,
    pub status: clicker::ConnectionStatus,
    pub log: VecDeque<clicker::LogMessage>,
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::sync::Mutex;

    use super::State;
    use crate::clicker::ConnectionStatus;
    use tokio::sync::mpsc;

    #[test]
    fn test_usage() {
        let (cmd_tx, _cmd_rx) = mpsc::unbounded_channel();
        let (_evt_tx, evt_rx) = mpsc::unbounded_channel();
        let _state = State {
            event_rx: Mutex::new(evt_rx),
            cmd_tx,
            contract_key: None,
            count: 42,
            status: ConnectionStatus::Connecting,
            log: VecDeque::new(),
        };
    }
}
