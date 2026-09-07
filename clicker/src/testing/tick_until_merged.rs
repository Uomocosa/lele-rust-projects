use bevy::prelude::*;

use crate::testing;

#[must_use]
pub fn tick_until_merged(app: &mut App, owner: u64, want: i32, max_ticks: u32) -> bool {
    for _ in 0..max_ticks {
        app.update();
        if testing::get_count(app, owner) >= want {
            return true;
        }
    }
    testing::get_count(app, owner) >= want
}

#[cfg(test)]
mod tests {
    use super::tick_until_merged;
    use crate::clicker;
    use crate::testing;
    use freenet_libp2p_bevy_plugin::{net_id, p2p, roster};

    #[test]
    fn test_usage() {
        let mut app = testing::fixture(1, "alpha");
        app.update();
        let remote = net_id::NetworkId::from_peer("remote-peer");
        app.world_mut().resource_mut::<roster::Roster>().add_entry(
            "alpha".to_string(),
            [9u8; 32],
            "remote-peer".to_string(),
        );
        app.world_mut()
            .spawn((clicker::Owner(remote), clicker::ClickCounter::default()));
        app.world_mut()
            .resource_mut::<p2p::Events<clicker::ClickDelta>>()
            .push(p2p::Event::Message {
                from: "remote-peer".to_string(),
                payload: clicker::ClickDelta {
                    owner: remote,
                    delta: 4,
                },
            });
        assert!(tick_until_merged(&mut app, *remote, 4, 5));
        assert!(!tick_until_merged(&mut app, *remote, 99, 2));
    }
}
