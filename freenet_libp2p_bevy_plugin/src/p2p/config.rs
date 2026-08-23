use std::sync::Mutex;

use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use super::config_new;
use super::config_take_event_rx;
use crate::p2p;

pub struct Config<T: p2p::Message> {
    pub cmd_tx: UnboundedSender<p2p::Command<T>>,
    pub event_rx: Mutex<Option<UnboundedReceiver<p2p::Event<T>>>>,
}

#[rustfmt::skip]
impl<T: p2p::Message> Config<T> {
    pub fn new(cmd_tx: UnboundedSender<p2p::Command<T>>, event_rx: UnboundedReceiver<p2p::Event<T>>) -> Self {
        config_new::new(cmd_tx, event_rx)
    }
    pub fn take_event_rx(&self) -> UnboundedReceiver<p2p::Event<T>> {
        config_take_event_rx::take_event_rx(self)
    }
}
// no test_usage necessary — thin delegates, exercised by plugin.rs test_usage
