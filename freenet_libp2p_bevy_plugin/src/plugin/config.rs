use std::sync::Mutex;

use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use super::config_new;
use super::config_take_event_rx;
use super::config_take_roster_rx;
use crate::net_id;
use crate::p2p;
use crate::roster;

pub struct Config<T: p2p::Message> {
    pub own_id: net_id::NetworkId,
    pub cmd_tx: UnboundedSender<p2p::Command<T>>,
    pub event_rx: Mutex<Option<UnboundedReceiver<p2p::Event<T>>>>,
    pub roster_rx: Mutex<Option<UnboundedReceiver<roster::Event>>>,
}

#[rustfmt::skip]
impl<T: p2p::Message> Config<T> {
    pub fn new(
        own_id: net_id::NetworkId,
        cmd_tx: UnboundedSender<p2p::Command<T>>,
        event_rx: UnboundedReceiver<p2p::Event<T>>,
        roster_rx: UnboundedReceiver<roster::Event>,
    ) -> Self {
        config_new::new(own_id, cmd_tx, event_rx, roster_rx)
    }
    pub fn take_event_rx(&self) -> UnboundedReceiver<p2p::Event<T>> {
        config_take_event_rx::take_event_rx(self)
    }
    pub fn take_roster_rx(&self) -> UnboundedReceiver<roster::Event> {
        config_take_roster_rx::take_roster_rx(self)
    }
}
// no test_usage necessary — thin delegates, exercised by plugin.rs test_usage
