use bevy::prelude::*;

use crate::clicker;

pub fn handle_increment_click(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<clicker::IncrementButton>)>,
    mut state: ResMut<clicker::State>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            clicker::increment(&mut state, 1);
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use tokio::sync::mpsc;

    use super::handle_increment_click;
    use crate::clicker;

    #[test]
    fn test_usage() {
        let mut app = App::new();

        let (tx, mut rx) = mpsc::unbounded_channel();
        let key = freenet_stdlib::prelude::ContractKey::from_params_and_code(
            freenet_stdlib::prelude::Parameters::from(Vec::new()),
            freenet_stdlib::prelude::ContractCode::from(Vec::new()),
        );
        app.insert_resource(clicker::State {
            event_rx: std::sync::Mutex::new(mpsc::unbounded_channel().1),
            cmd_tx: tx,
            contract_key: key,
            count: 5,
            connected: false,
        });

        app.world_mut()
            .spawn((clicker::IncrementButton, Interaction::Pressed));

        app.add_systems(Update, handle_increment_click);
        app.update();

        let Ok(cmd) = rx.try_recv() else {
            panic!("expected increment command");
        };
        match cmd {
            clicker::Command::Increment { count } => assert_eq!(count, 6),
        }
    }
}
