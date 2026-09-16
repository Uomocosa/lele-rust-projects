use bevy::prelude::*;

use freenet_libp2p_bevy_plugin::p2p;

use crate::clicker;
use crate::constants;

pub fn publish_snapshot(
    targets: Query<(&clicker::PlayerNo, &clicker::ClickCounter)>,
    global: Res<clicker::GlobalCounter>,
    lobby: Res<clicker::ActiveLobby>,
    tombstones: Res<clicker::ScoreTombstones>,
    commands: ResMut<p2p::Commands<clicker::CursorMsg>>,
) {
    let lobby = lobby.into_inner();
    let global = global.into_inner();
    let tombstones = tombstones.into_inner();
    let commands = commands.into_inner();
    let snapshot = clicker::Snapshot {
        entries: snapshot_entries(&targets, tombstones),
        global: **global,
    };
    commands.push(p2p::Command::PutHistory {
        lobby: (**lobby).clone(),
        chunk: constants::SNAPSHOT_CHUNK,
        data: clicker::encode_snapshot(&snapshot),
    });
}

// needed helper: merges live slots with parked tombstone scores, keeping the max per id
fn snapshot_entries(
    targets: &Query<(&clicker::PlayerNo, &clicker::ClickCounter)>,
    tombstones: &clicker::ScoreTombstones,
) -> Vec<(freenet_libp2p_bevy_plugin::net_id::NetworkId, i32)> {
    let mut entries: Vec<(freenet_libp2p_bevy_plugin::net_id::NetworkId, i32)> = targets
        .iter()
        .map(|(player, counter)| {
            (
                freenet_libp2p_bevy_plugin::net_id::NetworkId(**player),
                **counter,
            )
        })
        .collect();
    for (logical, count) in tombstones.iter() {
        let id = freenet_libp2p_bevy_plugin::net_id::NetworkId(*logical);
        let mut merged = false;
        for (known, known_count) in &mut entries {
            if *known == id {
                if *count > *known_count {
                    *known_count = *count;
                }
                merged = true;
                break;
            }
        }
        if !merged {
            entries.push((id, *count));
        }
    }
    entries
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::publish_snapshot;
    use crate::clicker;
    use freenet_libp2p_bevy_plugin::{net_id, p2p};

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(clicker::GlobalCounter::default());
        app.insert_resource(clicker::ActiveLobby("alpha".to_string()));
        app.insert_resource(clicker::ScoreTombstones::default());
        app.insert_resource(p2p::Commands::<clicker::CursorMsg>::default());
        app.world_mut()
            .spawn((clicker::PlayerNo(1), clicker::ClickCounter(4)));
        app.world_mut()
            .resource_mut::<clicker::ScoreTombstones>()
            .keep(2, 7);
        app.add_systems(Update, publish_snapshot);
        app.update();
        let commands = app.world().resource::<p2p::Commands<clicker::CursorMsg>>();
        assert_eq!(commands.len(), 1);
        let Some(p2p::Command::PutHistory { lobby, data, .. }) = commands.first() else {
            panic!("expected PutHistory");
        };
        assert_eq!(lobby, "alpha");
        let snapshot = clicker::decode_snapshot(data).unwrap();
        assert!(snapshot.entries.contains(&(net_id::NetworkId(1), 4)));
        assert!(snapshot.entries.contains(&(net_id::NetworkId(2), 7)));
    }

    #[test]
    fn tombstone_wins_over_stale_live_slot() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(clicker::GlobalCounter::default());
        app.insert_resource(clicker::ActiveLobby("alpha".to_string()));
        app.insert_resource(clicker::ScoreTombstones::default());
        app.insert_resource(p2p::Commands::<clicker::CursorMsg>::default());
        app.world_mut()
            .spawn((clicker::PlayerNo(2), clicker::ClickCounter(1)));
        app.world_mut()
            .resource_mut::<clicker::ScoreTombstones>()
            .keep(2, 7);
        app.add_systems(Update, publish_snapshot);
        app.update();
        let commands = app.world().resource::<p2p::Commands<clicker::CursorMsg>>();
        let Some(p2p::Command::PutHistory { data, .. }) = commands.first() else {
            panic!("expected PutHistory");
        };
        let snapshot = clicker::decode_snapshot(data).unwrap();
        assert!(snapshot.entries.contains(&(net_id::NetworkId(2), 7)));
    }
}
