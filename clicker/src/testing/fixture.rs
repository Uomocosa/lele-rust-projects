use bevy::prelude::*;
use freenet_libp2p_bevy_plugin::{net_id, p2p, roster};

use crate::clicker;

pub fn fixture(own: u64, lobby: &str) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<Time>();
    app.insert_resource(ButtonInput::<MouseButton>::default());
    app.insert_resource(p2p::Commands::<clicker::ClickDelta>::default());
    app.insert_resource(p2p::Events::<clicker::ClickDelta>::default());
    app.insert_resource(roster::Roster::default());
    app.insert_resource(net_id::NetworkId(own));
    app.insert_resource(clicker::AutoClick(false));
    app.insert_resource(clicker::GlobalCounter::default());
    app.insert_resource(clicker::ActiveLobby(lobby.to_string()));
    app.insert_resource(clicker::InstanceInfo {
        namespace: "test".to_string(),
        instance_tag: 0,
        own_id: net_id::NetworkId(own),
    });
    app.add_plugins(clicker::Plugin);
    app
}

#[cfg(test)]
mod tests {
    use super::fixture;

    #[test]
    fn test_usage() {
        let mut app = fixture(1, "alpha");
        app.update();
        app.update();
    }
}
