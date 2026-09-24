use tokio::sync::mpsc::UnboundedReceiver;

use crate::p2p;

pub fn take_rx(tap: &p2p::EventTap) -> Option<UnboundedReceiver<p2p::TapEvent>> {
    tap.rx.lock().ok()?.take()
}

#[cfg(test)]
mod tests {
    use super::take_rx;
    use crate::p2p;

    #[test]
    fn test_usage() {
        let tap = p2p::EventTap::default();
        assert!(take_rx(&tap).is_some());
        assert!(take_rx(&tap).is_none());
    }
}
