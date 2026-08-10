use std::sync::Mutex;

use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use super::config_new;
use super::config_take_evt_rx;
use crate::clicker;

pub struct Config {
    pub cmd_tx: UnboundedSender<clicker::Command>,
    pub(crate) evt_rx: Mutex<Option<UnboundedReceiver<clicker::Event>>>,
}

#[rustfmt::skip]
impl Config {
    pub fn new(
        cmd_tx: UnboundedSender<clicker::Command>,
        evt_rx: UnboundedReceiver<clicker::Event>,
    ) -> Self {
        config_new::new(cmd_tx, evt_rx)
    }
    pub fn take_evt_rx(&self) -> UnboundedReceiver<clicker::Event> {
        config_take_evt_rx::take_evt_rx(self)
    }
}
// no test_usage necessary
