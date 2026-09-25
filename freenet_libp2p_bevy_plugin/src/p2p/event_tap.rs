use std::sync::Mutex;

use atomic_delegate_macros::atomic_delegate;
use bevy::prelude::Resource;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use super::TapEvent;

#[derive(Resource)]
pub struct EventTap {
    pub tx: UnboundedSender<TapEvent>,
    pub rx: Mutex<Option<UnboundedReceiver<TapEvent>>>,
}

#[atomic_delegate]
impl EventTap {
    pub fn take_rx(&self) -> Option<UnboundedReceiver<TapEvent>> {}
}

impl EventTap {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for EventTap {
    fn default() -> Self {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        Self {
            tx,
            rx: Mutex::new(Some(rx)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::EventTap;
    use crate::p2p;

    #[test]
    fn test_usage() {
        let tap = EventTap::default();
        assert!(
            tap.tx
                .send(p2p::TapEvent::PeerConnected("p".to_string()))
                .is_ok()
        );
        assert!(tap.take_rx().is_some());
        assert!(tap.take_rx().is_none());
    }
}
