use bevy::prelude::App;
use derive_more::Deref;

use super::plugin_build;
use crate::p2p;
use crate::plugin;

#[derive(Deref)]
pub struct Plugin<T: p2p::Message>(pub plugin::Config<T>);

#[rustfmt::skip]
impl<T: p2p::Message> bevy::prelude::Plugin for Plugin<T> {
    fn build(&self, app: &mut App) { plugin_build::build(self, app) }
}

#[cfg(test)]
mod tests {
    use derive_more::Deref;
    use serde::{Deserialize, Serialize};

    use bevy::prelude::*;

    use super::Plugin;
    use crate::net_id;
    use crate::p2p;
    use crate::plugin;
    use crate::roster;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Deref)]
    struct Dummy(u32);

    #[test]
    fn test_usage() {
        let own_id = net_id::NetworkId(1);
        let (cmd_tx, _cmd_rx) = tokio::sync::mpsc::unbounded_channel();
        let (_event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel();
        let (_roster_tx, roster_rx) = tokio::sync::mpsc::unbounded_channel::<roster::Event>();
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::transform::TransformPlugin);
        app.insert_resource(ButtonInput::<KeyCode>::default());
        app.add_plugins(Plugin(plugin::Config::<Dummy>::new(
            own_id, cmd_tx, event_rx, roster_rx,
        )));
        app.update();

        assert!(
            app.world()
                .get_resource::<p2p::P2pCommands<Dummy>>()
                .is_some()
        );
        assert!(
            app.world()
                .get_resource::<p2p::P2pEvents<Dummy>>()
                .is_some()
        );
        assert!(app.world().get_resource::<roster::Roster>().is_some());
    }
}
