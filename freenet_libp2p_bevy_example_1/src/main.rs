use bevy::prelude::*;
use bevy_freenet::boxes;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(boxes::Plugin)
        .run();
}
