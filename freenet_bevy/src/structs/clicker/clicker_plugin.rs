use std::sync::Mutex;

use bevy::prelude::*;
use freenet_stdlib::prelude::ContractKey;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::structs::clicker::clicker_command::ClickerCommand;
use crate::structs::clicker::clicker_event::ClickerEvent;

pub struct ClickerPlugin {
    pub config: ClickerConfig,
}

pub struct ClickerConfig {
    pub cmd_tx: UnboundedSender<ClickerCommand>,
    evt_rx: Mutex<Option<UnboundedReceiver<ClickerEvent>>>,
    pub contract_key: ContractKey,
    pub initial_count: u64,
}

impl ClickerConfig {
    pub fn new(
        cmd_tx: UnboundedSender<ClickerCommand>,
        evt_rx: UnboundedReceiver<ClickerEvent>,
        contract_key: ContractKey,
        initial_count: u64,
    ) -> Self {
        Self {
            cmd_tx,
            evt_rx: Mutex::new(Some(evt_rx)),
            contract_key,
            initial_count,
        }
    }

    pub fn take_evt_rx(&self) -> UnboundedReceiver<ClickerEvent> {
        self.evt_rx
            .lock()
            .unwrap()
            .take()
            .expect("evt_rx already taken")
    }
}

#[rustfmt::skip]
impl Plugin for ClickerPlugin {
    fn build(&self, app: &mut App) {
        crate::methods::clicker::clicker_plugin::build::build(self, app)
    }
}
