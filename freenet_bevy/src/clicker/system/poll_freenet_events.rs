use bevy::prelude::*;

use crate::clicker::ClickerEvent;
use crate::clicker::message::CountChanged::CountChanged;
use crate::clicker::resource::State::ClickerState;

pub fn poll_freenet_events(
    state: ResMut<ClickerState>,
    mut count_writer: MessageWriter<CountChanged>,
) {
    let mut rx = state.event_rx.lock().unwrap();
    while let Ok(event) = rx.try_recv() {
        match event {
            ClickerEvent::Notification { count } => {
                drop(rx);
                let mut s = state;
                s.count = count;
                count_writer.write(CountChanged { count });
                return;
            }
            ClickerEvent::UpdateResponse { count } => {
                drop(rx);
                let mut s = state;
                s.count = count;
                count_writer.write(CountChanged { count });
                return;
            }
            ClickerEvent::Init { .. } => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::poll_freenet_events;
    use crate::clicker::ClickerEvent;
    use crate::clicker::message::CountChanged::CountChanged;
    use crate::clicker::resource::State::ClickerState;
    use tokio::sync::mpsc;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_message::<CountChanged>();

        let (evt_tx, evt_rx) = mpsc::unbounded_channel();
        let (cmd_tx, _cmd_rx) = mpsc::unbounded_channel();
        let key = freenet_stdlib::prelude::ContractKey::from_params_and_code(
            freenet_stdlib::prelude::Parameters::from(Vec::new()),
            freenet_stdlib::prelude::ContractCode::from(Vec::new()),
        );
        app.insert_resource(ClickerState {
            event_rx: std::sync::Mutex::new(evt_rx),
            cmd_tx,
            contract_key: key,
            count: 0,
        });

        app.add_systems(Update, poll_freenet_events);

        assert!(evt_tx.send(ClickerEvent::Notification { count: 7 }).is_ok());
        app.update();

        let state = app.world().resource::<ClickerState>();
        assert_eq!(state.count, 7);

        let count_msgs = app.world().resource::<Messages<CountChanged>>().len();
        assert_eq!(count_msgs, 1);
    }
}
