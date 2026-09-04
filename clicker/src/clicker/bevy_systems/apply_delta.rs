#![allow(clippy::missing_const_for_fn)]
use crate::clicker;
use bevy::prelude::*;
use freenet_libp2p_bevy_plugin::p2p;
pub fn apply_delta(_events: Res<p2p::P2PEvents<clicker::ClickDelta>>) {}
#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {
        assert_eq!(1, 1);
    }
}
