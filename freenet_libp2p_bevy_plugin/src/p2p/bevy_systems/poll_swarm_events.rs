use bevy::prelude::*;

use crate::p2p;

pub fn poll_swarm_events<T: p2p::Message>(
    mut events: ResMut<p2p::P2pEvents<T>>,
    mut peer_status: ResMut<p2p::PeerStatus>,
    mut incoming: MessageWriter<p2p::IncomingSnapshot<T>>,
    mut connected: MessageWriter<p2p::Connected>,
    mut disconnected: MessageWriter<p2p::Disconnected>,
) {
    while let Ok(event) = events.try_recv() {
        match event {
            p2p::Event::IncomingSnapshot { from, snapshot } => {
                incoming.write(p2p::IncomingSnapshot {
                    from: p2p::peer_id_to_network_id(&from),
                    snapshot,
                });
            }
            p2p::Event::PeerConnected(peer_id) => {
                peer_status.insert(peer_id.to_base58());
                connected.write(p2p::Connected(p2p::peer_id_to_network_id(&peer_id)));
            }
            p2p::Event::PeerDisconnected(peer_id) => {
                peer_status.remove(&peer_id.to_base58());
                disconnected.write(p2p::Disconnected(p2p::peer_id_to_network_id(&peer_id)));
            }
            p2p::Event::Error(reason) => {
                tracing::warn!(target: "p2p", reason, "p2p error");
            }
            p2p::Event::Ready { .. } => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use derive_more::Deref;
    use serde::{Deserialize, Serialize};

    use bevy::prelude::*;

    use super::poll_swarm_events;
    use crate::p2p;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Deref)]
    struct Dummy(u32);

    #[test]
    fn test_usage() {
        let (event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel::<p2p::Event<Dummy>>();
        let peer_id = libp2p::PeerId::random();
        let network_id = p2p::peer_id_to_network_id(&peer_id);
        let mut app = App::new();
        app.add_message::<p2p::IncomingSnapshot<Dummy>>();
        app.add_message::<p2p::Connected>();
        app.add_message::<p2p::Disconnected>();
        app.insert_resource(p2p::P2pEvents(event_rx));
        app.insert_resource(p2p::PeerStatus::default());
        app.add_systems(Update, poll_swarm_events::<Dummy>);

        event_tx
            .send(p2p::Event::IncomingSnapshot {
                from: peer_id,
                snapshot: p2p::Snapshot {
                    from_id: network_id,
                    tick: 1,
                    sent_at_ms: 0,
                    payload: Dummy(3),
                },
            })
            .ok();
        event_tx.send(p2p::Event::PeerConnected(peer_id)).ok();
        app.update();

        let status = app.world_mut().resource_mut::<p2p::PeerStatus>();
        assert!(status.contains(&peer_id.to_base58()));

        let mut incoming_reader = app
            .world_mut()
            .resource_mut::<bevy::ecs::message::Messages<p2p::IncomingSnapshot<Dummy>>>();
        assert_eq!(incoming_reader.drain().count(), 1);
    }

    #[test]
    fn emits_disconnected_for_peer() {
        let (event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel::<p2p::Event<Dummy>>();
        let peer_id = libp2p::PeerId::random();
        let network_id = p2p::peer_id_to_network_id(&peer_id);
        let mut app = App::new();
        app.add_message::<p2p::IncomingSnapshot<Dummy>>();
        app.add_message::<p2p::Connected>();
        app.add_message::<p2p::Disconnected>();
        app.insert_resource(p2p::P2pEvents(event_rx));
        app.insert_resource(p2p::PeerStatus::default());
        app.add_systems(Update, poll_swarm_events::<Dummy>);

        event_tx.send(p2p::Event::PeerDisconnected(peer_id)).ok();
        app.update();

        let mut disconnected_reader = app
            .world_mut()
            .resource_mut::<bevy::ecs::message::Messages<p2p::Disconnected>>();
        let got: Vec<_> = disconnected_reader.drain().collect();
        assert!(got.contains(&p2p::Disconnected(network_id)));
    }
}

// no test_usage necessary — real coverage is in tests above
