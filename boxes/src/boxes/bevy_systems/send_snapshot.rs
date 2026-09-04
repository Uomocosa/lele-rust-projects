#![allow(clippy::missing_const_for_fn)]
use crate::boxes;
use bevy::prelude::*;
use freenet_libp2p_bevy_plugin::p2p;
pub fn send_snapshot(_commands: ResMut<p2p::P2PCommands<boxes::Payload>>) {}
#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {
        assert_eq!(1, 1);
    }
}
