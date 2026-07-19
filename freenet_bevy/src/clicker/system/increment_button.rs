use bevy::prelude::*;

use crate::clicker::ClickerCommand;
use crate::clicker::component::IncrementButton::IncrementButton;
use crate::clicker::resource::State::ClickerState;

pub fn increment_button(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<IncrementButton>)>,
    mut state: ResMut<ClickerState>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            state.count = state.count.wrapping_add(1);
            let cmd = ClickerCommand::Increment { count: state.count };
            let _ = state.cmd_tx.send(cmd);
        }
    }
}
