use std::sync::Mutex;

use freenet_stdlib::prelude::ContractKey;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use super::config_new;
use super::config_take_evt_rx;
use crate::clicker;

pub struct Config {
    pub cmd_tx: UnboundedSender<clicker::Command>,
    pub(crate) evt_rx: Mutex<Option<UnboundedReceiver<clicker::Event>>>,
    pub contract_key: ContractKey,
    pub initial_count: u64,
}

#[rustfmt::skip]
impl Config {
    pub fn new(
        cmd_tx: UnboundedSender<clicker::Command>,
        evt_rx: UnboundedReceiver<clicker::Event>,
        contract_key: ContractKey,
        initial_count: u64,
    ) -> Self {
        config_new::new(cmd_tx, evt_rx, contract_key, initial_count)
    }
    pub fn take_evt_rx(&self) -> UnboundedReceiver<clicker::Event> {
        config_take_evt_rx::take_evt_rx(self)
    }
}
// no test_usage necessary
