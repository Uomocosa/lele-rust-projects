use std::collections::HashSet;

use bevy::prelude::*;

use freenet_libp2p_bevy_plugin::{net_id, p2p, roster};

use crate::clicker;
use crate::constants;
use crate::discovery;

pub fn absorb_roster(
    mut events: ResMut<p2p::Events<clicker::CursorMsg>>,
    commands: ResMut<p2p::Commands<clicker::CursorMsg>>,
    lobby: Res<clicker::ActiveLobby>,
    own: Res<net_id::NetworkId>,
    roster: ResMut<roster::Roster>,
    mut dialed: Local<HashSet<u64, std::hash::RandomState>>,
    mut presence: Local<HashSet<String, std::hash::RandomState>>,
) {
    let lobby = lobby.into_inner();
    let own = own.into_inner();
    let commands = commands.into_inner();
    let roster = roster.into_inner();
    let topic = clicker::gossip_roster_topic(lobby);
    let now = epoch_secs();
    let union = refresh_presence(&mut presence, roster);
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
                tracing::debug!("roster gossip lobby={} entries={}", **lobby, incoming.len());
                for (id, entry) in &incoming {
                    if let Some((peer_id, addrs)) =
                        absorb_entry(&mut dialed, **own, now, **id, entry)
                    {
                        if !presence.contains(&peer_id) || union.contains(&peer_id) {
                            let key = *blake3::hash(peer_id.as_bytes()).as_bytes();
                            roster.add_entry((**lobby).clone(), key, peer_id.clone());
                            tracing::debug!("roster gossip learned peer={peer_id}");
                        }
                        commands.push(p2p::Command::Dial { peer_id, addrs });
                    }
                }
            }
            other => rest.push(other),
        }
    }
    events.extend(rest);
}

// needed helper: decides whether a gossiped entry needs a dial, dialing each id once
fn absorb_entry(
    dialed: &mut HashSet<u64, std::hash::RandomState>,
    own: u64,
    now: u64,
    id: u64,
    entry: &discovery::PeerEntry,
) -> Option<(String, Vec<String>)> {
    if id == own || entry.addrs.is_empty() {
        return None;
    }
    if now.saturating_sub(entry.updated_at) > constants::STALE_ENTRY_SECS {
        tracing::debug!("roster gossip stale peer={} skipped", entry.peer_id);
        return None;
    }
    if !dialed.insert(id) {
        return None;
    }
    Some((entry.peer_id.clone(), entry.addrs.clone()))
}

// needed helper: records the peers present in any lobby and returns them, so gossip never resurrects vanished members
fn refresh_presence(
    presence: &mut HashSet<String, std::hash::RandomState>,
    roster: &roster::Roster,
) -> HashSet<String, std::hash::RandomState> {
    let union: HashSet<String, std::hash::RandomState> = roster
        .values()
        .flat_map(|members| members.values().cloned())
        .collect();
    for peer in &union {
        presence.insert(peer.clone());
    }
    union
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
    use freenet_libp2p_bevy_plugin::{net_id, p2p, roster};

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
        app.insert_resource(roster::Roster::default());
        let mut state = discovery::RosterState::new();
        state.insert(discovery::PlayerId(3), entry("peer-3", u64::MAX));
        app.world_mut()
            .resource_mut::<p2p::Events<clicker::CursorMsg>>()
            .push(gossip_state(&state));
        app.add_systems(Update, absorb_roster);
        app.update();
        let commands = app.world().resource::<p2p::Commands<clicker::CursorMsg>>();
        assert_eq!(commands.len(), 1);
        let members = app
            .world()
            .resource::<roster::Roster>()
            .get("alpha")
            .map_or(0, std::collections::BTreeMap::len);
        assert_eq!(members, 1);
        app.update();
        let commands = app.world().resource::<p2p::Commands<clicker::CursorMsg>>();
        assert_eq!(commands.len(), 1);
    }

    #[test]
    fn connected_peer_never_redialed() {
        use super::epoch_secs;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(p2p::Events::<clicker::CursorMsg>::default());
        app.insert_resource(p2p::Commands::<clicker::CursorMsg>::default());
        app.insert_resource(clicker::ActiveLobby("alpha".to_string()));
        app.insert_resource(net_id::NetworkId(1));
        app.insert_resource(roster::Roster::default());
        let mut state = discovery::RosterState::new();
        state.insert(discovery::PlayerId(3), entry("peer-3", epoch_secs()));
        app.world_mut()
            .resource_mut::<p2p::Events<clicker::CursorMsg>>()
            .push(gossip_state(&state));
        app.add_systems(Update, absorb_roster);
        app.update();
        let commands = app.world().resource::<p2p::Commands<clicker::CursorMsg>>();
        assert_eq!(commands.len(), 1);
        app.world_mut()
            .resource_mut::<p2p::Events<clicker::CursorMsg>>()
            .push(p2p::Event::PeerConnected("peer-3".to_string()));
        app.update();
        let mut heartbeat = discovery::RosterState::new();
        heartbeat.insert(
            discovery::PlayerId(3),
            entry("peer-3", epoch_secs().saturating_add(100)),
        );
        app.world_mut()
            .resource_mut::<p2p::Events<clicker::CursorMsg>>()
            .push(gossip_state(&heartbeat));
        app.update();
        let commands = app.world().resource::<p2p::Commands<clicker::CursorMsg>>();
        assert_eq!(commands.len(), 1);
    }
}
