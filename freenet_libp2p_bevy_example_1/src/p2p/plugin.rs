use bevy::prelude::App;
use derive_more::Deref;

use super::plugin_build;
use crate::p2p;

#[derive(Deref)]
pub struct Plugin(pub p2p::Config);

#[rustfmt::skip]
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) { plugin_build::build(self, app) }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::Plugin;
    use crate::boxes;
    use crate::p2p;
    use crate::roster;

    #[test]
    fn test_usage() {
        let (cmd_tx, _cmd_rx) = tokio::sync::mpsc::unbounded_channel();
        let (_event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel();
        let mut app = App::new();
        app.insert_resource(boxes::Config::new(boxes::PlayerId(1)));
        app.insert_resource(roster::Roster(roster::RosterState::new()));
        app.add_plugins(Plugin(p2p::Config::new(cmd_tx, event_rx)));
        app.update();

        assert!(app.world().get_resource::<p2p::P2pCommands>().is_some());
        assert!(app.world().get_resource::<p2p::P2pEvents>().is_some());
    }
}
