use bevy::app::AppExit;
use bevy::prelude::*;

use crate::clicker::headless::State::HeadlessState;
use crate::clicker::message::CountChanged::CountChanged;
use crate::clicker::resource::State::ClickerState;

pub fn headless_counter(
    state: Res<ClickerState>,
    mut count_reader: MessageReader<CountChanged>,
    mut hs: ResMut<HeadlessState>,
    mut app_exit: MessageWriter<AppExit>,
) {
    for _ in count_reader.read() {
        hs.pending = false;
        hs.completed += 1;
        println!("tick {}, count={}", hs.completed, state.count);
        if hs.completed >= hs.max_ticks {
            app_exit.write(AppExit::Success);
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::app::AppExit;
    use bevy::prelude::*;

    use super::headless_counter;
    use crate::clicker::headless::State::HeadlessState;
    use crate::clicker::message::CountChanged::CountChanged;
    use crate::clicker::resource::State::ClickerState;
    use tokio::sync::mpsc;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_message::<CountChanged>();

        let (cmd_tx, _cmd_rx) = mpsc::unbounded_channel();
        let (_evt_tx, evt_rx) = mpsc::unbounded_channel();
        let key = freenet_stdlib::prelude::ContractKey::from_params_and_code(
            freenet_stdlib::prelude::Parameters::from(Vec::new()),
            freenet_stdlib::prelude::ContractCode::from(Vec::new()),
        );
        app.insert_resource(ClickerState {
            event_rx: std::sync::Mutex::new(evt_rx),
            cmd_tx,
            contract_key: key,
            count: 42,
        });
        app.insert_resource(HeadlessState {
            max_ticks: 2,
            pending: false,
            completed: 0,
        });

        app.add_systems(Update, headless_counter);

        // Write a CountChanged event
        app.world_mut()
            .resource_mut::<Messages<CountChanged>>()
            .write(CountChanged { count: 1 });

        app.update();

        let hs = app.world().resource::<HeadlessState>();
        assert_eq!(hs.completed, 1);
        assert!(!hs.pending);

        // Write another event
        app.world_mut()
            .resource_mut::<Messages<CountChanged>>()
            .write(CountChanged { count: 2 });

        app.update();

        let hs = app.world().resource::<HeadlessState>();
        assert_eq!(hs.completed, 2);

        // AppExit should be written
        let exits = app.world().resource::<Messages<AppExit>>().len();
        assert_eq!(exits, 1);
    }
}
