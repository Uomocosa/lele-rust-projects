use bevy::prelude::*;

use freenet_libp2p_bevy_plugin::net_id;
use freenet_libp2p_bevy_plugin::p2p;

use crate::clicker;

pub fn absorb_snapshot(
    mut events: ResMut<p2p::Events<clicker::CursorMsg>>,
    mut targets: Query<(&clicker::Owner, &mut clicker::ClickCounter)>,
    mut global: ResMut<clicker::GlobalCounter>,
    lobby: Res<clicker::ActiveLobby>,
    own: Res<net_id::NetworkId>,
) {
    let own = own.into_inner();
    let lobby = lobby.into_inner();
    let mut rest = Vec::new();
    for event in events.take_all() {
        match event {
            p2p::Event::HistoryChunk {
                lobby: chunk_lobby,
                chunk,
                data,
            } if chunk_lobby == **lobby && chunk == clicker::SNAPSHOT_CHUNK => {
                let Some(snapshot) = clicker::decode_snapshot(&data) else {
                    continue;
                };
                for (id, count) in snapshot.entries {
                    if id == *own {
                        continue;
                    }
                    for (owner, mut counter) in &mut targets {
                        if **owner == id {
                            let current = **counter;
                            if count > current {
                                counter.add(count.saturating_sub(current));
                            }
                            break;
                        }
                    }
                }
                let current = **global;
                if snapshot.global > current {
                    global.add(snapshot.global.saturating_sub(current));
                }
            }
            other => rest.push(other),
        }
    }
    events.extend(rest);
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::absorb_snapshot;
    use crate::clicker;
    use freenet_libp2p_bevy_plugin::{net_id, p2p};

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(p2p::Events::<clicker::CursorMsg>::default());
        app.insert_resource(clicker::GlobalCounter::default());
        app.insert_resource(clicker::ActiveLobby("alpha".to_string()));
        app.insert_resource(net_id::NetworkId(1));
        let remote = net_id::NetworkId::from_peer("remote-peer");
        let target = app
            .world_mut()
            .spawn((clicker::Owner(remote), clicker::ClickCounter(2)))
            .id();
        let snapshot = clicker::Snapshot {
            entries: vec![(remote, 5)],
            global: 9,
        };
        app.world_mut()
            .resource_mut::<p2p::Events<clicker::CursorMsg>>()
            .push(p2p::Event::HistoryChunk {
                lobby: "alpha".to_string(),
                chunk: clicker::SNAPSHOT_CHUNK,
                data: clicker::encode_snapshot(&snapshot),
            });
        app.world_mut()
            .resource_mut::<p2p::Events<clicker::CursorMsg>>()
            .push(p2p::Event::HistoryChunk {
                lobby: "other".to_string(),
                chunk: clicker::SNAPSHOT_CHUNK,
                data: clicker::encode_snapshot(&snapshot),
            });
        app.add_systems(Update, absorb_snapshot);
        app.update();
        assert_eq!(
            **app.world().get::<clicker::ClickCounter>(target).unwrap(),
            5
        );
        assert_eq!(**app.world().resource::<clicker::GlobalCounter>(), 9);
        assert_eq!(
            app.world()
                .resource::<p2p::Events<clicker::CursorMsg>>()
                .len(),
            1
        );
    }
}
