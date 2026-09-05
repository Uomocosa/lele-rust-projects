use crate::clicker;
use bevy::prelude::*;
pub fn build(_plugin: &clicker::Plugin, app: &mut App) {
    app.add_systems(Startup, clicker::bevy_systems::setup)
        .add_systems(Update, clicker::bevy_systems::detect_click)
        .add_systems(Update, clicker::bevy_systems::spawn_on_join)
        .add_systems(Update, clicker::bevy_systems::despawn_on_leave)
        .add_systems(Update, clicker::bevy_systems::apply_delta)
        .add_systems(Update, clicker::bevy_systems::render);
}

#[cfg(test)]
mod tests {
    use super::build;
    use bevy::prelude::*;
    use freenet_libp2p_bevy_plugin::{p2p, roster};

    use crate::clicker;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(p2p::Commands::<clicker::ClickDelta>::default());
        app.insert_resource(p2p::Events::<clicker::ClickDelta>::default());
        app.insert_resource(roster::Roster::default());
        build(&clicker::Plugin, &mut app);
        app.update();
    }
}
