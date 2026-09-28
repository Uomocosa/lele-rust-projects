//! UI preview host for the `lobby_room` status line.
//!
//! Runs the exact `lobby_room` status UI with no freenet node, no P2P and no
//! discovery plugin, so nothing overwrites `discovery::Multiplayer`. An external
//! driver (`lele-ui-preview`) injects `Multiplayer` fixtures over the Bevy Remote
//! Protocol (`world.insert_resources`) and captures `brp_extras/screenshot`.
//! The BRP port comes from `BRP_EXTRAS_PORT` (default 15702, bound to localhost).

#[path = "../lobby_room/status.rs"]
mod status;

use bevy::prelude::*;
use bevy_brp_extras::BrpExtrasPlugin;
use freenet_libp2p_bevy_plugin::discovery;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "lobby-preview".to_string(),
            resolution: (960, 540).into(),
            ..default()
        }),
        ..default()
    }));
    app.add_plugins(BrpExtrasPlugin::new());
    app.init_resource::<discovery::Multiplayer>();
    app.insert_resource(status::Username("preview".to_string()));
    app.add_systems(Startup, status::setup_ui);
    app.add_systems(Update, status::log_tick);
    app.run();
}
