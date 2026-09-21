use std::collections::HashMap;

use bevy::prelude::*;
use freenet_libp2p_bevy_plugin::{net_id, p2p, roster};

use crate::clicker;

pub fn tick_presence(
    time: Res<Time>,
    mut events: ResMut<p2p::Events<clicker::CursorMsg>>,
    mut presence: Local<HashMap<u64, f64, std::hash::RandomState>>,
    mut roster: ResMut<roster::Roster>,
    lobby: Res<clicker::ActiveLobby>,
) {
    let now = time.into_inner().elapsed().as_secs_f64();
    let lobby = lobby.into_inner();
    let mut rest = Vec::new();
    for event in events.take_all() {
        match &event {
            p2p::Event::PeerDisconnected(peer) => {
                presence
                    .entry(*net_id::NetworkId::from_peer(peer))
                    .or_insert(now);
            }
            p2p::Event::Message { from: peer, .. } | p2p::Event::Gossip { from: peer, .. } => {
                presence.remove(&*net_id::NetworkId::from_peer(peer));
            }
            _ => {}
        }
        rest.push(event);
    }
    events.extend(rest);
    let room = (**lobby).clone();
    let expired: Vec<String> = {
        let mut found = Vec::new();
        if let Some(members) = roster.get(&room) {
            for peer in members.values() {
                let owner = *net_id::NetworkId::from_peer(peer.as_str());
                if presence
                    .get(&owner)
                    .is_some_and(|since| now - *since >= clicker::PRESENCE_TTL_SECS)
                {
                    found.push(peer.clone());
                }
            }
        }
        found
    };
    for peer in expired {
        let key = *blake3::hash(peer.as_bytes()).as_bytes();
        roster.remove_entry(&room, key);
        presence.remove(&*net_id::NetworkId::from_peer(peer.as_str()));
        tracing::info!(target: "clicker", peer = %peer, "presence: dead peer removed");
        clicker::DecisionLog::record(&format!("presence: dead peer removed peer={peer}"));
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use freenet_libp2p_bevy_plugin::{net_id, p2p, roster};

    use super::tick_presence;
    use crate::clicker;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Time>();
        app.insert_resource(p2p::Events::<clicker::CursorMsg>::default());
        app.insert_resource(roster::Roster::default());
        app.insert_resource(clicker::ActiveLobby("alpha".to_string()));
        app.add_systems(Update, tick_presence);
        app
    }

    fn add_member(app: &mut App, peer: &str) {
        app.world_mut().resource_mut::<roster::Roster>().add_entry(
            "alpha".to_string(),
            *blake3::hash(peer.as_bytes()).as_bytes(),
            peer.to_string(),
        );
    }

    #[test]
    fn test_usage() {
        let mut app = test_app();
        app.update();
        assert!(app.world().resource::<roster::Roster>().is_empty());
    }

    #[test]
    fn disconnected_peer_removed_after_ttl() {
        let mut app = test_app();
        add_member(&mut app, "peer-3");
        app.world_mut()
            .resource_mut::<p2p::Events<clicker::CursorMsg>>()
            .push(p2p::Event::PeerDisconnected("peer-3".to_string()));
        app.update();
        assert!(
            app.world()
                .resource::<roster::Roster>()
                .get("alpha")
                .is_some_and(|members| members.len() == 1),
            "removal waits out the presence TTL"
        );
        app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs(1),
        ));
        for _ in 0..80 {
            app.update();
        }
        assert!(
            app.world()
                .resource::<roster::Roster>()
                .get("alpha")
                .is_none_or(std::collections::BTreeMap::is_empty),
            "dead peer leaves the roster after the TTL"
        );
    }

    #[test]
    fn liveness_clears_the_suspect() {
        let mut app = test_app();
        add_member(&mut app, "peer-3");
        app.world_mut()
            .resource_mut::<p2p::Events<clicker::CursorMsg>>()
            .push(p2p::Event::PeerDisconnected("peer-3".to_string()));
        app.update();
        app.world_mut()
            .resource_mut::<p2p::Events<clicker::CursorMsg>>()
            .push(p2p::Event::Message {
                from: "peer-3".to_string(),
                payload: clicker::CursorMsg::Move {
                    owner: net_id::NetworkId(3),
                    pos: [0.0, 0.0],
                },
            });
        app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs(1),
        ));
        for _ in 0..80 {
            app.update();
        }
        assert!(
            app.world()
                .resource::<roster::Roster>()
                .get("alpha")
                .is_some_and(|members| members.len() == 1),
            "a live peer is never pruned"
        );
    }
}
