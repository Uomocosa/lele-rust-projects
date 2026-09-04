#![allow(clippy::missing_const_for_fn)]
use bevy::prelude::*;
use freenet_libp2p_bevy_plugin::roster;
pub fn despawn_on_leave(_roster: Res<roster::Roster>) {}
#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {
        assert_eq!(1, 1);
    }
}
