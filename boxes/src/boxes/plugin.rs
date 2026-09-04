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
    use freenet_libp2p_bevy_plugin::{net_id, p2p, roster};

    fn install_network_resources(app: &mut App) {
        app.insert_resource(p2p::P2PCommands::<boxes::Payload>::default());
        app.insert_resource(p2p::P2PEvents::<boxes::Payload>::default());
        app.insert_resource(roster::Roster::default());
    }

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::transform::TransformPlugin);
        app.insert_resource(ButtonInput::<KeyCode>::default());
        install_network_resources(&mut app);
        app.add_plugins(Plugin(boxes::Config::new(net_id::NetworkId(1))));
        app.update();

        let config = app.world().get_resource::<boxes::Config>();
        assert_eq!(config.map(|c| **c), Some(net_id::NetworkId(1)));
    }
}
