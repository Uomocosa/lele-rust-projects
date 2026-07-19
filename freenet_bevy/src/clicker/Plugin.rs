use std::sync::Mutex;

use bevy::prelude::*;
use freenet_stdlib::prelude::ContractKey;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::clicker::clicker_command::ClickerCommand;
use crate::clicker::clicker_event::ClickerEvent;
use crate::clicker::headless::State::HeadlessConfig;

pub struct ClickerPlugin {
    pub config: ClickerConfig,
}

pub struct ClickerConfig {
    pub cmd_tx: UnboundedSender<ClickerCommand>,
    evt_rx: Mutex<Option<UnboundedReceiver<ClickerEvent>>>,
    pub contract_key: ContractKey,
    pub initial_count: u64,
    pub headless: Option<HeadlessConfig>,
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
            headless: None,
        }
    }

    pub fn with_headless(mut self, config: HeadlessConfig) -> Self {
        self.headless = Some(config);
        self
    }

    pub fn take_evt_rx(&self) -> UnboundedReceiver<ClickerEvent> {
        self.evt_rx
            .lock()
            .unwrap()
            .take()
            .expect("evt_rx already taken")
    }
}

impl Plugin for ClickerPlugin {
    fn build(&self, app: &mut App) {
        crate::clicker::PluginMethod::build(self, app);
    }
}
