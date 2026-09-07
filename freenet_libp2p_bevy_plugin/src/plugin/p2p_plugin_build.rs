use std::sync::Mutex;

use bevy::prelude::*;

use super::p2p_plugin::P2PPlugin;
use crate::p2p;
use crate::roster;

pub fn build<T: p2p::Message>(plugin: &P2PPlugin<T>, app: &mut App) {
    let own_id = plugin.own_id;
    let cmd_tx = plugin.cmd_tx.clone();
    app.insert_resource(own_id);
    app.insert_resource(p2p::Events::<T>::default());
    app.insert_resource(p2p::Commands::<T>::default());
    app.insert_resource(roster::Roster::default());
    let lobby: roster::Lobby = roster::Lobby::default();
    app.insert_resource(lobby);
    if let Some(event_rx) = plugin.take_event_rx() {
        app.insert_resource(p2p::Bridge {
            cmd_tx,
            event_rx: Mutex::new(Some(event_rx)),
        });
    }
    app.add_systems(
        Update,
        (
            p2p::bevy_systems::poll_p2p_events::<T>,
            roster::bevy_systems::poll_roster::<T>,
        )
            .chain(),
    );
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use serde::{Deserialize, Serialize};

    use super::build;
    use crate::net_id;
    use crate::p2p;
    use crate::plugin;
    use crate::roster;
    use derive_more::Deref;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default, Deref)]
    struct Dummy(u32);

    #[test]
    fn test_usage() {
        let own_id = net_id::NetworkId(1);
        let (cmd_tx, _) = tokio::sync::mpsc::unbounded_channel();
        let (_, event_rx) = tokio::sync::mpsc::unbounded_channel();
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        build(
            &plugin::P2PPlugin(plugin::Config::<Dummy>::new(own_id, cmd_tx, event_rx)),
            &mut app,
        );
        app.update();
        assert!(app.world().get_resource::<p2p::Events<Dummy>>().is_some());
        assert!(app.world().get_resource::<p2p::Bridge<Dummy>>().is_some());
        assert!(app.world().get_resource::<roster::Roster>().is_some());
        assert_eq!(
            app.world().get_resource::<roster::Lobby>(),
            Some(&roster::Lobby("default".to_string()))
        );
        assert!(app.world().get_resource::<net_id::NetworkId>().is_some());
    }
}
