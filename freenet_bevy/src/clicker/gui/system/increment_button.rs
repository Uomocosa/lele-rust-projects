use bevy::prelude::*;

use crate::clicker::ClickerCommand;
use crate::clicker::gui::component::IncrementButton;
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

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::increment_button;
    use crate::clicker::ClickerCommand;
    use crate::clicker::gui::component::IncrementButton;
    use crate::clicker::resource::State::ClickerState;
    use tokio::sync::mpsc;

    #[test]
    fn test_usage() {
        let mut app = App::new();

        let (tx, mut rx) = mpsc::unbounded_channel();
        let key = freenet_stdlib::prelude::ContractKey::from_params_and_code(
            freenet_stdlib::prelude::Parameters::from(Vec::new()),
            freenet_stdlib::prelude::ContractCode::from(Vec::new()),
        );
        app.insert_resource(ClickerState {
            event_rx: std::sync::Mutex::new(mpsc::unbounded_channel().1),
            cmd_tx: tx,
            contract_key: key,
            count: 5,
        });

        app.world_mut()
            .spawn((IncrementButton, Interaction::Pressed));

        app.add_systems(Update, increment_button);
        app.update();

        let Ok(cmd) = rx.try_recv() else {
            panic!("expected increment command");
        };
        match cmd {
            ClickerCommand::Increment { count } => assert_eq!(count, 6),
        }
    }
}
