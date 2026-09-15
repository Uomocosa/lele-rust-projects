use std::collections::HashMap;

use bevy::prelude::*;

use freenet_libp2p_bevy_plugin::{net_id, p2p};

use crate::clicker;
use crate::constants;
use crate::discovery;

pub fn absorb_roster(
    mut events: ResMut<p2p::Events<clicker::CursorMsg>>,
    commands: ResMut<p2p::Commands<clicker::CursorMsg>>,
    lobby: Res<clicker::ActiveLobby>,
    own: Res<net_id::NetworkId>,
    mut dialed: Local<HashMap<u64, u64, std::hash::RandomState>>,
) {
    let lobby = lobby.into_inner();
    let own = own.into_inner();
    let commands = commands.into_inner();
    let topic = clicker::gossip_roster_topic(lobby);
    let now = epoch_secs();
    let mut rest = Vec::new();
    for event in events.take_all() {
        match event {
            p2p::Event::Gossip {
                topic: incoming,
                data,
                ..
            } if incoming == topic => {
                let incoming: discovery::RosterState =
                    bincode::deserialize(&data).unwrap_or_default();
                for (id, entry) in &incoming {
                    absorb_entry(commands, &mut dialed, **own, now, **id, entry);
                }
            }
            other => rest.push(other),
        }
    }
    events.extend(rest);
}

// needed helper: dials one gossiped entry unless stale, own, or already dialled fresh
fn absorb_entry(
    commands: &mut p2p::Commands<clicker::CursorMsg>,
    dialed: &mut HashMap<u64, u64, std::hash::RandomState>,
    own: u64,
    now: u64,
    id: u64,
    entry: &discovery::PeerEntry,
) {
    if id == own || entry.addrs.is_empty() {
        return;
    }
    if now.saturating_sub(entry.updated_at) > constants::STALE_ENTRY_SECS {
        return;
    }
    if dialed
        .get(&id)
        .is_some_and(|known| *known >= entry.updated_at)
    {
        return;
    }
    dialed.insert(id, entry.updated_at);
    commands.push(p2p::Command::Dial {
        peer_id: entry.peer_id.clone(),
        addrs: entry.addrs.clone(),
    });
}

// needed helper: seconds since the unix epoch for staleness checks
fn epoch_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::absorb_roster;
    use crate::clicker;
    use crate::discovery;
    use freenet_libp2p_bevy_plugin::{net_id, p2p};

    fn gossip_state(state: &discovery::RosterState) -> p2p::Event<clicker::CursorMsg> {
        p2p::Event::Gossip {
            topic: clicker::gossip_roster_topic(&clicker::ActiveLobby("alpha".to_string())),
            from: "peer-2".to_string(),
            data: bincode::serialize(state).unwrap_or_default(),
        }
    }

    fn entry(peer_id: &str, updated_at: u64) -> discovery::PeerEntry {
        discovery::PeerEntry {
            peer_id: peer_id.to_string(),
            addrs: vec!["/ip4/127.0.0.1/tcp/4001".to_string()],
            updated_at,
        }
    }

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(p2p::Events::<clicker::CursorMsg>::default());
        app.insert_resource(p2p::Commands::<clicker::CursorMsg>::default());
        app.insert_resource(clicker::ActiveLobby("alpha".to_string()));
        app.insert_resource(net_id::NetworkId(1));
        let mut state = discovery::RosterState::new();
        state.insert(discovery::PlayerId(3), entry("peer-3", u64::MAX));
        app.world_mut()
            .resource_mut::<p2p::Events<clicker::CursorMsg>>()
            .push(gossip_state(&state));
        app.add_systems(Update, absorb_roster);
        app.update();
        let commands = app.world().resource::<p2p::Commands<clicker::CursorMsg>>();
        assert_eq!(commands.len(), 1);
        app.update();
        let commands = app.world().resource::<p2p::Commands<clicker::CursorMsg>>();
        assert_eq!(commands.len(), 1);
    }
}
