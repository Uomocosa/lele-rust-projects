use bevy::prelude::App;
use derive_more::Deref;

use super::plugin_build;
use crate::boxes;

#[derive(Deref)]
pub struct Plugin(pub boxes::Config);

#[rustfmt::skip]
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) { plugin_build::build(self, app) }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::Plugin;
    use crate::boxes;
    use crate::engine;
    use crate::p2p;
    use crate::roster;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::transform::TransformPlugin);
        app.insert_resource(ButtonInput::<KeyCode>::default());
        let (cmd_tx, _cmd_rx) = tokio::sync::mpsc::unbounded_channel::<p2p::Command>();
        let (_event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel::<p2p::Event>();
        app.insert_resource(p2p::P2pCommands(cmd_tx));
        app.insert_resource(p2p::P2pEvents(event_rx));
        app.insert_resource(roster::Roster(roster::RosterState::new()));
        app.add_plugins(Plugin(boxes::Config::new([1; 32])));
        app.update();

        let config = app.world().get_resource::<boxes::Config>();
        assert_eq!(config.map(|c| **c), Some([1; 32]));
        assert!(app.world().get_resource::<engine::EngineHandle>().is_some());
        assert!(
            app.world()
                .get_resource::<boxes::LatestSnapshot>()
                .is_some()
        );
        assert!(
            app.world()
                .get_resource::<boxes::NetcodeLockstep>()
                .is_some()
        );
    }
}
