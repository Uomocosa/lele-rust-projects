use std::sync::Mutex;

use bevy::app::App;
use bevy::prelude::*;

use crate::clicker::headless::State::HeadlessState;
use crate::clicker::headless::system;
use crate::clicker::message::CountChanged::CountChanged;
use crate::clicker::plugin::ClickerPlugin;
use crate::clicker::resource::State::ClickerState;

pub fn build(plugin: &ClickerPlugin, app: &mut App) {
    app.add_message::<CountChanged>();
    let evt_rx = plugin.config.take_evt_rx();
    app.insert_resource(ClickerState {
        event_rx: Mutex::new(evt_rx),
        cmd_tx: plugin.config.cmd_tx.clone(),
        contract_key: plugin.config.contract_key,
        count: plugin.config.initial_count,
    });

    if let Some(ref hc) = plugin.config.headless {
        app.insert_resource(HeadlessState {
            max_ticks: hc.max_ticks,
            pending: false,
            completed: 0,
        });
        app.add_systems(
            Update,
            (
                (
                    crate::clicker::system::poll_freenet_events::poll_freenet_events,
                    system::headless_counter::headless_counter,
                )
                    .chain(),
                system::headless_tick::headless_tick,
            ),
        );
    } else {
        app.add_systems(Startup, crate::clicker::system::spawn_ui::spawn_ui);
        app.add_systems(
            Update,
            (
                crate::clicker::system::poll_freenet_events::poll_freenet_events,
                crate::clicker::system::increment_button::increment_button,
                crate::clicker::system::update_counter_ui::update_counter_ui,
            ),
        );
    }
}
