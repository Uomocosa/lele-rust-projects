use bevy::prelude::*;
use freenet_libp2p_bevy_plugin::roster;
pub const fn spawn_on_join(_roster: Res<roster::Roster>) {}
#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {
        assert_eq!(1, 1);
    }
}
