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

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::headless_tick;
    use crate::clicker::ClickerCommand;
    use crate::clicker::headless::State::HeadlessState;
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
            count: 10,
        });
        app.insert_resource(HeadlessState {
            max_ticks: 5,
            pending: false,
            completed: 0,
        });

        app.add_systems(Update, headless_tick);
        app.update();

        // Should send an increment command
        let Ok(cmd) = rx.try_recv() else {
            panic!("expected increment command");
        };
        match cmd {
            ClickerCommand::Increment { count } => assert_eq!(count, 11),
        }

        // pending should be true
        let hs = app.world().resource::<HeadlessState>();
        assert!(hs.pending);

        // Second update should not send (pending is true)
        app.update();
        assert!(rx.try_recv().is_err());
    }
}
