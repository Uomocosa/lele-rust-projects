use bevy::prelude::Resource;
use derive_more::Deref;
use tokio::sync::mpsc::UnboundedSender;

use super::net_command::NetCommand;

#[derive(Resource, Deref)]
pub struct NetBridge(pub UnboundedSender<NetCommand>);

#[cfg(test)]
mod tests {
    use super::NetBridge;
    use crate::p2p;

    #[test]
    fn test_usage() {
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel::<p2p::NetCommand>();
        let bridge = NetBridge(tx);
        assert!(!bridge.is_closed());
    }
}
