use bevy::app::App;
use bevy::prelude::*;

use crate::roster;

pub fn build(plugin: &roster::Plugin, app: &mut App) {
    let event_rx = plugin.take_event_rx();

    app.insert_resource(roster::Roster::default())
        .insert_resource(roster::RosterEvents(event_rx))
        .insert_resource(roster::FreenetStatus::default())
        .add_systems(
            Update,
            (
                roster::bevy_systems::poll_freenet_events,
                roster::bevy_systems::spawn_roster_boxes,
                roster::bevy_systems::despawn_roster_boxes,
                roster::bevy_systems::update_status_text,
            )
                .chain(),
        );
}
// no test_usage necessary — exercised by plugin.rs test_usage
