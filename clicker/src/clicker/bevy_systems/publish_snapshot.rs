use bevy::prelude::*;

use freenet_libp2p_bevy_plugin::p2p;

use crate::clicker;

pub fn publish_snapshot(
    time: Res<Time>,
    mut published: Local<bool>,
    mut last: Local<f64>,
    targets: Query<(&clicker::Owner, &clicker::ClickCounter)>,
    global: Res<clicker::GlobalCounter>,
    lobby: Res<clicker::ActiveLobby>,
    commands: ResMut<p2p::Commands<clicker::CursorMsg>>,
) {
    let time = time.into_inner();
    let now = time.elapsed().as_secs_f64();
    if *published && now - *last < clicker::SNAPSHOT_INTERVAL_SECS {
        return;
    }
    *published = true;
    *last = now;
    let lobby = lobby.into_inner();
    let global = global.into_inner();
    let commands = commands.into_inner();
    let snapshot = clicker::Snapshot {
        entries: targets
            .iter()
            .map(|(owner, counter)| (**owner, **counter))
            .collect(),
        global: **global,
    };
    commands.push(p2p::Command::PutHistory {
        lobby: (**lobby).clone(),
        chunk: clicker::SNAPSHOT_CHUNK,
        data: clicker::encode_snapshot(&snapshot),
    });
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
        app.init_resource::<Time>();
        app.insert_resource(clicker::GlobalCounter::default());
        app.insert_resource(clicker::ActiveLobby("alpha".to_string()));
        app.insert_resource(p2p::Commands::<clicker::CursorMsg>::default());
        app.world_mut().spawn((
            clicker::Owner(net_id::NetworkId(1)),
            clicker::ClickCounter(4),
        ));
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
        app.update();
        let commands = app.world().resource::<p2p::Commands<clicker::CursorMsg>>();
        assert_eq!(commands.len(), 1);
    }
}
