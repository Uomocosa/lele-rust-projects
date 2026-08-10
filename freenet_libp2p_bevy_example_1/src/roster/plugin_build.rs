use bevy::app::App;
use bevy::prelude::*;

use crate::roster;

pub fn build(plugin: &roster::Plugin, app: &mut App) {
    let event_rx = plugin.take_event_rx();

    app.insert_resource(roster::Roster::default())
        .insert_resource(roster::RosterEvents(event_rx))
        .add_systems(
            Update,
            (
                roster::bevy_systems::poll_freenet_events,
                roster::bevy_systems::spawn_roster_boxes,
            )
                .chain(),
        );
}
// no test_usage necessary — exercised by plugin.rs test_usage
