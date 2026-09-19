use freenet_libp2p_bevy_plugin::p2p;

use super::mesh::Mesh;
use crate::clicker;

pub fn push_to(mesh: &mut Mesh, owner: u64, event: p2p::Event<clicker::CursorMsg>) -> bool {
    for app in &mut mesh.apps {
        let own = *app.world().resource::<clicker::InstanceInfo>().own_id;
        if own == owner {
            app.world_mut()
                .resource_mut::<p2p::Events<clicker::CursorMsg>>()
                .push(event);
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use freenet_libp2p_bevy_plugin::p2p;

    use super::push_to;
    use crate::testing;

    #[test]
    fn test_usage() {
        let mut mesh = testing::Mesh::of(3);
        let sent = push_to(
            &mut mesh,
            1,
            p2p::Event::PeerConnected("peer-2".to_string()),
        );
        assert!(sent);
        assert!(!push_to(
            &mut mesh,
            9,
            p2p::Event::PeerConnected("x".to_string())
        ));
    }
}
