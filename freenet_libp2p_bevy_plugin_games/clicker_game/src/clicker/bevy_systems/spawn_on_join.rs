use bevy::prelude::*;

use freenet_libp2p_bevy_plugin::{net_id, roster};

use crate::clicker;

pub fn spawn_on_join(
    mut joins: MessageReader<roster::PeerJoined>,
    mut commands: Commands,
    mut slot: Local<usize>,
    identity: Res<net_id::LocalIdentity>,
) {
    for join in joins.read() {
        let id = **join;
        if id == **identity {
            continue;
        }
        *slot += 1;
        clicker::spawn_target(&mut commands, id, *slot as i32, false);
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::spawn_on_join;
    use crate::clicker;
    use freenet_libp2p_bevy_plugin::{net_id, roster};

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_message::<roster::PeerJoined>();
        app.insert_resource(net_id::LocalIdentity(net_id::NetworkId(99)));
        app.world_mut()
            .resource_mut::<bevy::ecs::message::Messages<roster::PeerJoined>>()
            .write(roster::PeerJoined(net_id::NetworkId(7)));
        app.add_systems(Update, spawn_on_join);
        app.update();

        let mut query = app.world_mut().query::<&clicker::Owner>();
        let owners: Vec<_> = query.iter(app.world()).collect();
        assert_eq!(owners.len(), 1);
        assert_eq!(**owners[0], net_id::NetworkId(7));
    }
}
