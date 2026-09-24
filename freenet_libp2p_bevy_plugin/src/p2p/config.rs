use std::sync::Mutex;

use atomic_delegate_macros::atomic_delegate;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::net_id;
use crate::p2p;

pub struct Config<T: p2p::Message> {
    pub own_id: net_id::NetworkId,
    pub cmd_tx: UnboundedSender<p2p::Command<T>>,
    pub event_rx: Mutex<Option<UnboundedReceiver<p2p::Event<T>>>>,
}

#[atomic_delegate]
impl<T: p2p::Message> Config<T> {
    pub fn take_event_rx(&self) -> Option<UnboundedReceiver<p2p::Event<T>>> {}
}

impl<T: p2p::Message> Config<T> {
    #[must_use]
    pub const fn new(
        own_id: net_id::NetworkId,
        cmd_tx: UnboundedSender<p2p::Command<T>>,
        event_rx: UnboundedReceiver<p2p::Event<T>>,
    ) -> Self {
        Self {
            own_id,
            cmd_tx,
            event_rx: Mutex::new(Some(event_rx)),
        }
    }
}
// no test_usage necessary
