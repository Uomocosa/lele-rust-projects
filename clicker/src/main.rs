use bevy::prelude::*;
use clicker_lib::clicker;

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(clicker::GamePlugin::new())
        .run();
}
