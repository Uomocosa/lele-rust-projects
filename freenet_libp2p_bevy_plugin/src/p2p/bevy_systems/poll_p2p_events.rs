use bevy::prelude::*;

use crate::p2p;

pub fn poll_p2p_events<T: p2p::Message>(
    mut events: ResMut<p2p::Events<T>>,
    mut commands: ResMut<p2p::Commands<T>>,
    mut outbox: ResMut<p2p::Outbox>,
    bridge: Res<p2p::Bridge<T>>,
    net_bridge: Option<Res<p2p::NetBridge>>,
    signals: Option<Res<p2p::Signals>>,
    tap: Option<Res<p2p::EventTap>>,
) {
    let bridge = bridge.into_inner();
    for command in commands.take_all() {
        bridge.cmd_tx.send(command).ok();
    }
    if let Some(net_bridge) = net_bridge {
        for command in outbox.take_all() {
            net_bridge.send(command).ok();
        }
    }
    let drained = bridge.event_rx.lock().map_or_else(
        |_| Vec::new(),
        |mut guard| {
            let mut drained = Vec::new();
            if let Some(rx) = guard.as_mut() {
                while let Ok(event) = rx.try_recv() {
                    drained.push(event);
                }
            }
            drained
        },
    );
    if let Some(signals) = signals {
        for event in &drained {
            publish_signal(&signals, event);
        }
    }
    if let Some(tap) = tap {
        for event in &drained {
            if let Some(tapped) = tap_of(event) {
                tap.tx.send(tapped).ok();
            }
        }
    }
    events.extend(drained);
}

// needed helper: updates the latest-value ready/observed watch channels
fn publish_signal<T: p2p::Message>(signals: &p2p::Signals, event: &p2p::Event<T>) {
    match event {
        p2p::Event::Ready { peer_id, addrs } => {
            signals.ready_tx.send_replace(Some(p2p::Ready {
                peer_id: peer_id.clone(),
                addrs: addrs.clone(),
            }));
        }
        p2p::Event::ObservedAddr(addr) => {
            signals.observed_tx.send_replace(Some(vec![addr.clone()]));
        }
        _ => {}
    }
}

// needed helper: maps the non-generic subset of events onto the discovery tap
fn tap_of<T: p2p::Message>(event: &p2p::Event<T>) -> Option<p2p::TapEvent> {
    match event {
        p2p::Event::Ready { peer_id, addrs } => Some(p2p::TapEvent::Ready {
            peer_id: peer_id.clone(),
            addrs: addrs.clone(),
        }),
        p2p::Event::ObservedAddr(addr) => Some(p2p::TapEvent::ObservedAddr(addr.clone())),
        p2p::Event::PeerConnected(peer) => Some(p2p::TapEvent::PeerConnected(peer.clone())),
        p2p::Event::PeerDisconnected(peer) => Some(p2p::TapEvent::PeerDisconnected(peer.clone())),
        p2p::Event::DialFailed { peer_id, reason } => Some(p2p::TapEvent::DialFailed {
            peer_id: peer_id.clone(),
            reason: reason.clone(),
        }),
        p2p::Event::LobbyProviders { lobby, peers } => Some(p2p::TapEvent::LobbyProviders {
            lobby: lobby.clone(),
            peers: peers.clone(),
        }),
        p2p::Event::Gossip { topic, from, data } => Some(p2p::TapEvent::Gossip {
            topic: topic.clone(),
            from: from.clone(),
            data: data.clone(),
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use derive_more::Deref;
    use serde::{Deserialize, Serialize};

    use super::poll_p2p_events;
    use crate::p2p;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Deref)]
    struct Dummy(u32);

    #[test]
    fn test_usage() {
        let (cmd_tx, mut cmd_rx) = tokio::sync::mpsc::unbounded_channel();
        let (event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel::<p2p::Event<Dummy>>();
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(p2p::Events::<Dummy>::default());
        app.insert_resource(p2p::Commands::<Dummy>::default());
        app.insert_resource(p2p::Outbox::default());
        event_tx
            .send(p2p::Event::PeerConnected("peer".to_string()))
            .ok();
        app.world_mut()
            .resource_mut::<p2p::Commands<Dummy>>()
            .push(p2p::Command::Dial {
                peer_id: "peer".to_string(),
                addrs: vec![],
            });
        app.insert_resource(p2p::Bridge {
            cmd_tx,
            event_rx: std::sync::Mutex::new(Some(event_rx)),
        });
        app.add_systems(Update, poll_p2p_events::<Dummy>);
        app.update();
        assert!(cmd_rx.try_recv().is_ok());
        assert_eq!(app.world().resource::<p2p::Events<Dummy>>().len(), 1);
    }

    #[test]
    fn tap_forwards_non_generic_events() {
        let (cmd_tx, _cmd_rx) = tokio::sync::mpsc::unbounded_channel();
        let (event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel::<p2p::Event<Dummy>>();
        let (tap_tx, mut tap_rx) = tokio::sync::mpsc::unbounded_channel::<p2p::TapEvent>();
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(p2p::Events::<Dummy>::default());
        app.insert_resource(p2p::Commands::<Dummy>::default());
        app.insert_resource(p2p::Outbox::default());
        app.insert_resource(p2p::Bridge {
            cmd_tx,
            event_rx: std::sync::Mutex::new(Some(event_rx)),
        });
        app.insert_resource(p2p::EventTap {
            tx: tap_tx,
            rx: std::sync::Mutex::new(None),
        });
        event_tx
            .send(p2p::Event::PeerConnected("peer".to_string()))
            .ok();
        event_tx
            .send(p2p::Event::Message {
                from: "peer".to_string(),
                payload: Dummy(1),
            })
            .ok();
        app.add_systems(Update, poll_p2p_events::<Dummy>);
        app.update();
        let tapped = tap_rx.try_recv();
        assert_eq!(tapped, Ok(p2p::TapEvent::PeerConnected("peer".to_string())));
        assert!(tap_rx.try_recv().is_err(), "game messages stay off the tap");
    }
}
