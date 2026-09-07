use bevy::prelude::*;

use crate::p2p;

pub fn poll_p2p_events<T: p2p::Message>(
    mut events: ResMut<p2p::Events<T>>,
    mut commands: ResMut<p2p::Commands<T>>,
    bridge: Res<p2p::Bridge<T>>,
) {
    let bridge = bridge.into_inner();
    for command in commands.take_all() {
        bridge.cmd_tx.send(command).ok();
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
    events.extend(drained);
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
}
