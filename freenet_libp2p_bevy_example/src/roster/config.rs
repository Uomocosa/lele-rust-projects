use std::sync::Mutex;

use derive_more::Deref;
use tokio::sync::mpsc::UnboundedReceiver;

use super::config_new;
use super::config_take_event_rx;
use crate::roster;

#[derive(Deref)]
pub struct Config(pub(crate) Mutex<Option<UnboundedReceiver<roster::Event>>>);

#[rustfmt::skip]
impl Config {
    pub fn new(event_rx: UnboundedReceiver<roster::Event>) -> Self { config_new::new(event_rx) }
    pub fn take_event_rx(&self) -> UnboundedReceiver<roster::Event> { config_take_event_rx::take_event_rx(self) }
}
// no test_usage necessary
