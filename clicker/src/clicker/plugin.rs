use super::plugin_build;
use bevy::prelude::App;

pub struct Plugin;
#[rustfmt::skip]
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) { plugin_build::build(self, app) }
}
#[cfg(test)]
mod tests {
    use super::Plugin;
    use bevy::prelude::*;
    use freenet_libp2p_bevy_plugin::{net_id, p2p, roster};

    use crate::clicker;
    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Time>();
        app.insert_resource(ButtonInput::<MouseButton>::default());
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<ColorMaterial>>();
        app.insert_resource(p2p::Commands::<clicker::CursorMsg>::default());
        app.insert_resource(p2p::Events::<clicker::CursorMsg>::default());
        app.insert_resource(roster::Roster::default());
        app.insert_resource(net_id::NetworkId(1));
        app.insert_resource(clicker::GlobalCounter::default());
        app.insert_resource(clicker::PendingClicks::default());
        app.insert_resource(clicker::ActiveLobby("alpha".to_string()));
        app.insert_resource(clicker::InstanceInfo {
            namespace: "test".to_string(),
            instance_tag: 0,
            own_id: net_id::NetworkId(1),
        });
        app.add_plugins(Plugin);
        app.update();
    }
}
