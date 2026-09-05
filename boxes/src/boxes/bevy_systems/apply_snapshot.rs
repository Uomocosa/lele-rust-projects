use crate::boxes;
use bevy::prelude::*;
use freenet_libp2p_bevy_plugin::p2p;
pub const fn apply_snapshot(_events: Res<p2p::Events<boxes::Payload>>) {}
#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {
        assert_eq!(1, 1);
    }
}
