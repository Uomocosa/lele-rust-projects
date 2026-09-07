use bevy::prelude::*;

use crate::p2p;
use crate::roster;

pub fn poll_roster<T: p2p::Message>(
    mut roster: ResMut<roster::Roster>,
    lobby: Res<roster::Lobby>,
    mut events: ResMut<p2p::Events<T>>,
) {
    let mut rest = Vec::new();
    let lobby = lobby.into_inner();
    for event in events.take_all() {
        match event {
            p2p::Event::PeerConnected(peer) => {
                let id = *blake3::hash(peer.as_bytes()).as_bytes();
                roster.add_entry((**lobby).clone(), id, peer);
            }
            p2p::Event::PeerDisconnected(peer) => {
                let id = *blake3::hash(peer.as_bytes()).as_bytes();
                roster.remove_entry(lobby, id);
            }
            p2p::Event::Ready { peer_id, addrs } => {
                tracing::info!("ready peer_id={peer_id} addrs={addrs:?}");
            }
            p2p::Event::Error(message) => {
                tracing::warn!("p2p error: {message}");
            }
            other => rest.push(other),
        }
    }
    events.extend(rest);
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use bevy::prelude::*;
    use derive_more::Deref;
    use serde::{Deserialize, Serialize};

    use super::poll_roster;
    use crate::p2p;
    use crate::roster;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Deref)]
    struct Dummy(u32);

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(roster::Roster::default());
        app.insert_resource(roster::Lobby("alpha".to_string()));
        app.insert_resource(p2p::Events::<Dummy>::default());
        app.world_mut()
            .resource_mut::<p2p::Events<Dummy>>()
            .push(p2p::Event::PeerConnected("peer".to_string()));
        app.world_mut()
            .resource_mut::<p2p::Events<Dummy>>()
            .push(p2p::Event::Message {
                from: "peer".to_string(),
                payload: Dummy(1),
            });
        app.add_systems(Update, poll_roster::<Dummy>);
        app.update();
        let alpha = app
            .world()
            .resource::<roster::Roster>()
            .get("alpha")
            .map_or(0, std::collections::BTreeMap::len);
        assert_eq!(alpha, 1);
        assert!(
            app.world()
                .resource::<roster::Roster>()
                .get("default")
                .is_none()
        );
        app.world_mut()
            .resource_mut::<p2p::Events<Dummy>>()
            .push(p2p::Event::PeerDisconnected("peer".to_string()));
        app.update();
        let members: usize = app
            .world()
            .resource::<roster::Roster>()
            .values()
            .map(BTreeMap::len)
            .sum();
        assert_eq!(members, 0);
        assert_eq!(app.world().resource::<p2p::Events<Dummy>>().len(), 1);
    }
}
