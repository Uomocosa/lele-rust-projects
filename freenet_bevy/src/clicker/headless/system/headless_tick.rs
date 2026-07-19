use bevy::prelude::*;

use crate::clicker::headless::State::HeadlessState;
use crate::clicker::resource::State::ClickerState;

pub fn headless_tick(state: Res<ClickerState>, mut hs: ResMut<HeadlessState>) {
    if hs.completed >= hs.max_ticks || hs.pending {
        return;
    }
    hs.pending = true;
    let count = state.count.wrapping_add(1);
    tracing::info!(target: "freenet_bevy", count, "headless tick sending increment");
    let _ = state
        .cmd_tx
        .send(crate::clicker::ClickerCommand::Increment { count });
}
