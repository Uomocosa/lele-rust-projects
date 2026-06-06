use bevy::prelude::*;

use crate::Plugin;
use crate::resource::FreenetNode;

pub fn build(plugin: &Plugin, app: &mut App) {
    app.init_resource::<FreenetNode>();
    app.add_message::<crate::Event>();
    app.add_systems(FixedUpdate, crate::system::poll_events::poll_events);
    let _ = plugin;
}
