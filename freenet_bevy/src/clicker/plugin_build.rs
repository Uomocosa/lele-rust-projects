use std::sync::Mutex;

use bevy::app::App;
use bevy::prelude::*;

use crate::clicker::count_changed::CountChanged;
use crate::clicker::plugin::Plugin;
use crate::clicker::state::State;

pub fn build(plugin: &Plugin, app: &mut App) {
    app.add_message::<CountChanged>();
    let evt_rx = plugin.config.take_evt_rx();
    app.insert_resource(State {
        event_rx: Mutex::new(evt_rx),
        cmd_tx: plugin.config.cmd_tx.clone(),
        contract_key: plugin.config.contract_key,
        count: plugin.config.initial_count,
    });

    app.add_systems(
        Update,
        crate::clicker::systems::poll_freenet_events::poll_freenet_events,
    );
}
