#![allow(clippy::missing_const_for_fn)]
use crate::clicker;
use bevy::prelude::*;
use freenet_libp2p_bevy_plugin::p2p;
pub fn detect_click(_commands: ResMut<p2p::P2PCommands<clicker::ClickDelta>>) {}
#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {
        assert_eq!(1, 1);
    }
}
