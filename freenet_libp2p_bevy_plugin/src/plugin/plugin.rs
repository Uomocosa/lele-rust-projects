use bevy::prelude::App;
use derive_more::Deref;

use super::plugin_build;
use crate::p2p;
use crate::plugin;

#[derive(Deref)]
pub struct P2PPlugin<T: p2p::Message>(pub plugin::Config<T>);

impl<T: p2p::Message> bevy::prelude::Plugin for P2PPlugin<T> {
    fn build(&self, app: &mut App) {
        let event_rx = self.take_event_rx();
        let _ = event_rx;
        app.insert_resource(p2p::P2PEvents::<T>::default());
        app.insert_resource(p2p::P2PCommands::<T>::default());
        app.insert_resource(crate::roster::Roster::default());
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use serde::{Deserialize, Serialize};

    use super::P2PPlugin;
    use crate::net_id;
    use crate::p2p;
    use crate::plugin;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
    struct Dummy(u32);

    #[test]
    fn test_usage() {
        let own_id = net_id::NetworkId(1);
        let (cmd_tx, _) = tokio::sync::mpsc::unbounded_channel();
        let (_, event_rx) = tokio::sync::mpsc::unbounded_channel();
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(P2PPlugin(plugin::Config::<Dummy>::new(
            own_id, cmd_tx, event_rx,
        )));
        app.update();
        assert!(
            app.world()
                .get_resource::<p2p::P2PEvents<Dummy>>()
                .is_some()
        );
    }
}
