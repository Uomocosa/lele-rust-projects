use derive_more::Deref;

use super::config_new;
use crate::net_id;
use crate::p2p;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

#[derive(Deref)]
pub struct Config<T: p2p::Message>(pub p2p::Config<T>);

#[rustfmt::skip]
impl<T: p2p::Message> Config<T> {
    #[must_use]
    pub const fn new(
        own_id: net_id::NetworkId,
        cmd_tx: UnboundedSender<p2p::Command<T>>,
        event_rx: UnboundedReceiver<p2p::Event<T>>,
    ) -> Self {
        config_new::new(own_id, cmd_tx, event_rx)
    }
}
// no test_usage necessary
