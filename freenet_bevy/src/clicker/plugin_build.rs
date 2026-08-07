use std::sync::Mutex;

use bevy::app::App;
use bevy::prelude::*;

use crate::clicker;

pub fn build(plugin: &clicker::Plugin, app: &mut App) {
    app.add_message::<clicker::CountChanged>();
    let evt_rx = plugin.config.take_evt_rx();
    app.insert_resource(clicker::State {
        event_rx: Mutex::new(evt_rx),
        cmd_tx: plugin.config.cmd_tx.clone(),
        contract_key: plugin.config.contract_key,
        count: plugin.config.initial_count,
    });

    app.add_systems(Update, clicker::bevy_systems::poll_freenet_events);
}
// no test_usage necessary
