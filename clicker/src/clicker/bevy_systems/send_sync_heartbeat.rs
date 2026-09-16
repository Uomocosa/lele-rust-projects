use bevy::prelude::*;

use freenet_libp2p_bevy_plugin::{net_id, p2p, roster};

use crate::clicker;

pub fn send_sync_heartbeat(
    time: Res<Time>,
    mut last: Local<f64>,
    mut counts: Local<std::collections::HashMap<String, usize, std::hash::RandomState>>,
    roster: Res<roster::Roster>,
    lobby: Res<clicker::ActiveLobby>,
    commands: ResMut<p2p::Commands<clicker::CursorMsg>>,
    own: Res<net_id::NetworkId>,
) {
    let time = time.into_inner();
    let now = time.elapsed().as_secs_f64();
    let roster = roster.into_inner();
    let lobby = lobby.into_inner();
    let own = own.into_inner();
    let commands = commands.into_inner();
    let Some(members) = roster.get(&**lobby) else {
        return;
    };
    let changed = counts
        .get(&**lobby)
        .is_none_or(|known| *known != members.len());
    if !changed && now - *last < clicker::SYNC_INTERVAL_SECS {
        return;
    }
    *last = now;
    counts.insert((**lobby).clone(), members.len());
    for peer in members.values() {
        commands.push(p2p::Command::Send {
            peer_id: peer.clone(),
            payload: clicker::CursorMsg::SyncReq { requester: *own },
        });
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::send_sync_heartbeat;
    use crate::clicker;
    use freenet_libp2p_bevy_plugin::{net_id, p2p, roster};

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Time>();
        let mut roster = roster::Roster::default();
        roster.add_entry("alpha".to_string(), [2u8; 32], "peer-2".to_string());
        roster.add_entry("alpha".to_string(), [3u8; 32], "peer-3".to_string());
        app.insert_resource(roster);
        app.insert_resource(clicker::ActiveLobby("alpha".to_string()));
        app.insert_resource(p2p::Commands::<clicker::CursorMsg>::default());
        app.insert_resource(net_id::NetworkId(1));
        app.add_systems(Update, send_sync_heartbeat);
        for _ in 0..3 {
            app.update();
        }
        let commands = app.world().resource::<p2p::Commands<clicker::CursorMsg>>();
        assert!(
            commands.len() >= 2,
            "heartbeat syncs each member: {}",
            commands.len()
        );
    }

    #[test]
    fn growth_fires_without_waiting_for_interval() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Time>();
        app.insert_resource(roster::Roster::default());
        app.insert_resource(clicker::ActiveLobby("alpha".to_string()));
        app.insert_resource(p2p::Commands::<clicker::CursorMsg>::default());
        app.insert_resource(net_id::NetworkId(1));
        app.add_systems(Update, send_sync_heartbeat);
        app.update();
        let commands = app.world().resource::<p2p::Commands<clicker::CursorMsg>>();
        assert_eq!(commands.len(), 0, "empty roster sends nothing");
        app.world_mut().resource_mut::<roster::Roster>().add_entry(
            "alpha".to_string(),
            [2u8; 32],
            "peer-2".to_string(),
        );
        app.world_mut().resource_mut::<roster::Roster>().add_entry(
            "alpha".to_string(),
            [3u8; 32],
            "peer-3".to_string(),
        );
        app.update();
        let commands = app.world().resource::<p2p::Commands<clicker::CursorMsg>>();
        assert_eq!(
            commands.len(),
            2,
            "roster growth syncs at once, ignoring the interval"
        );
    }
}
