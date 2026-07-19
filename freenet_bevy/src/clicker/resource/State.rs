use std::sync::Mutex;

use bevy::prelude::Resource;
use freenet_stdlib::prelude::ContractKey;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::clicker::ClickerCommand;
use crate::clicker::ClickerEvent;

#[derive(Resource)]
pub struct ClickerState {
    pub event_rx: Mutex<UnboundedReceiver<ClickerEvent>>,
    pub cmd_tx: UnboundedSender<ClickerCommand>,
    pub contract_key: ContractKey,
    pub count: u64,
}
