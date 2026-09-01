use std::sync::Mutex;

use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use super::config_new;
use super::config_take_event_rx;
use crate::p2p;

pub struct Config {
    pub cmd_tx: UnboundedSender<p2p::Command>,
    pub event_rx: Mutex<Option<UnboundedReceiver<p2p::Event>>>,
}

#[rustfmt::skip]
impl Config {
    pub fn new(cmd_tx: UnboundedSender<p2p::Command>, event_rx: UnboundedReceiver<p2p::Event>) -> Self {
        config_new::new(cmd_tx, event_rx)
    }
    pub fn take_event_rx(&self) -> UnboundedReceiver<p2p::Event> {
        config_take_event_rx::take_event_rx(self)
    }
}
// no test_usage necessary — thin delegates, exercised by plugin.rs test_usage
