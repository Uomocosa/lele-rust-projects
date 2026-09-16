use bevy::prelude::*;

use freenet_libp2p_bevy_plugin::p2p;

use crate::clicker;
use crate::lobby;

pub fn poll_expected(
    feed: Res<lobby::ExpectedRx>,
    gate: ResMut<lobby::JoinGate>,
    events: ResMut<p2p::Events<clicker::CursorMsg>>,
) {
    let feed = feed.into_inner();
    let Ok(mut guard) = feed.lock() else {
        return;
    };
    let Some(rx) = guard.as_mut() else {
        return;
    };
    let gate = gate.into_inner();
    let events = events.into_inner();
    while let Ok(peers) = rx.try_recv() {
        gate.expected = Some(peers.iter().cloned().collect());
        for peer in &peers {
            events.push(p2p::Event::PeerConnected(peer.clone()));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::poll_expected;
    use crate::clicker;
    use crate::lobby;
    use bevy::prelude::*;
    use freenet_libp2p_bevy_plugin::p2p;
    use std::sync::Mutex;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<Vec<String>>();
        app.insert_resource(lobby::ExpectedRx(Mutex::new(Some(rx))));
        app.insert_resource(lobby::JoinGate::default());
        app.insert_resource(p2p::Events::<clicker::CursorMsg>::default());
        tx.send(vec!["peer-2".to_string(), "peer-3".to_string()])
            .ok();
        app.add_systems(Update, poll_expected);
        app.update();
        let gate = app.world().resource::<lobby::JoinGate>();
        assert!(
            gate.expected
                .as_ref()
                .is_some_and(|set| { set.contains("peer-2") && set.contains("peer-3") })
        );
    }

    #[test]
    fn latest_snapshot_wins() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<Vec<String>>();
        app.insert_resource(lobby::ExpectedRx(Mutex::new(Some(rx))));
        app.insert_resource(lobby::JoinGate::default());
        app.insert_resource(p2p::Events::<clicker::CursorMsg>::default());
        tx.send(vec!["peer-2".to_string()]).ok();
        tx.send(vec!["peer-4".to_string()]).ok();
        app.add_systems(Update, poll_expected);
        app.update();
        let gate = app.world().resource::<lobby::JoinGate>();
        assert!(
            gate.expected
                .as_ref()
                .is_some_and(|set| { set.len() == 1 && set.contains("peer-4") })
        );
    }

    #[test]
    fn expected_arrival_announces_known_peers() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<Vec<String>>();
        app.insert_resource(lobby::ExpectedRx(Mutex::new(Some(rx))));
        app.insert_resource(lobby::JoinGate::default());
        app.insert_resource(p2p::Events::<clicker::CursorMsg>::default());
        tx.send(vec!["peer-2".to_string(), "peer-3".to_string()])
            .ok();
        app.add_systems(Update, poll_expected);
        app.update();
        let events = app.world().resource::<p2p::Events<clicker::CursorMsg>>();
        let announced: Vec<String> = events
            .iter()
            .filter_map(|event| match event {
                p2p::Event::PeerConnected(peer) => Some(peer.clone()),
                _ => None,
            })
            .collect();
        assert!(
            announced.contains(&"peer-2".to_string()) && announced.contains(&"peer-3".to_string()),
            "expected peers surface as connections even when the link predates the room"
        );
    }
}
