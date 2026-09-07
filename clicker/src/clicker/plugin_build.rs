use crate::clicker;
use bevy::prelude::*;
pub fn build(_plugin: &clicker::Plugin, app: &mut App) {
    app.add_systems(Startup, clicker::bevy_systems::setup)
        .add_systems(Startup, clicker::bevy_systems::log_connected)
        .add_systems(Startup, clicker::bevy_systems::request_snapshot)
        .add_systems(Update, clicker::bevy_systems::detect_click)
        .add_systems(Update, clicker::bevy_systems::spawn_on_join)
        .add_systems(Update, clicker::bevy_systems::despawn_on_leave)
        .add_systems(Update, clicker::bevy_systems::apply_delta)
        .add_systems(Update, clicker::bevy_systems::absorb_snapshot)
        .add_systems(Update, clicker::bevy_systems::publish_snapshot)
        .add_systems(Update, clicker::bevy_systems::request_snapshot)
        .add_systems(Update, clicker::bevy_systems::render)
        .add_systems(Update, clicker::bevy_systems::update_score)
        .add_systems(Update, clicker::bevy_systems::tick_log)
        .add_systems(Update, clicker::bevy_systems::auto_tick);
}

#[cfg(test)]
mod tests {
    use super::build;
    use bevy::prelude::*;
    use freenet_libp2p_bevy_plugin::{net_id, p2p, roster};

    use crate::clicker;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Time>();
        app.insert_resource(ButtonInput::<MouseButton>::default());
        app.insert_resource(p2p::Commands::<clicker::ClickDelta>::default());
        app.insert_resource(p2p::Events::<clicker::ClickDelta>::default());
        app.insert_resource(roster::Roster::default());
        app.insert_resource(net_id::NetworkId(1));
        app.insert_resource(clicker::AutoClick(false));
        app.insert_resource(clicker::GlobalCounter::default());
        app.insert_resource(clicker::ActiveLobby("alpha".to_string()));
        app.insert_resource(clicker::InstanceInfo {
            namespace: "test".to_string(),
            instance_tag: 0,
            own_id: net_id::NetworkId(1),
        });
        build(&clicker::Plugin, &mut app);
        app.update();
    }
}
