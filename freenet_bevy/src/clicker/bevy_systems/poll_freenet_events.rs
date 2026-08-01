use bevy::prelude::*;

use crate::clicker;

pub fn poll_freenet_events(
    state: ResMut<clicker::State>,
    mut count_writer: MessageWriter<clicker::CountChanged>,
) {
    let mut rx = state.event_rx.lock().unwrap();
    while let Ok(event) = rx.try_recv() {
        match event {
            clicker::Event::Notification { count } => {
                drop(rx);
                let mut s = state;
                s.count = count;
                count_writer.write(clicker::CountChanged { count });
                return;
            }
            clicker::Event::UpdateResponse { count } => {
                drop(rx);
                let mut s = state;
                s.count = count;
                count_writer.write(clicker::CountChanged { count });
                return;
            }
            clicker::Event::Init { .. } => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::poll_freenet_events;
    use crate::clicker;
    use tokio::sync::mpsc;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_message::<clicker::CountChanged>();

        let (evt_tx, evt_rx) = mpsc::unbounded_channel();
        let (cmd_tx, _cmd_rx) = mpsc::unbounded_channel();
        let key = freenet_stdlib::prelude::ContractKey::from_params_and_code(
            freenet_stdlib::prelude::Parameters::from(Vec::new()),
            freenet_stdlib::prelude::ContractCode::from(Vec::new()),
        );
        app.insert_resource(clicker::State {
            event_rx: std::sync::Mutex::new(evt_rx),
            cmd_tx,
            contract_key: key,
            count: 0,
        });

        app.add_systems(Update, poll_freenet_events);

        assert!(
            evt_tx
                .send(clicker::Event::Notification { count: 7 })
                .is_ok()
        );
        app.update();

        let state = app.world().resource::<clicker::State>();
        assert_eq!(state.count, 7);

        let count_msgs = app
            .world()
            .resource::<Messages<clicker::CountChanged>>()
            .len();
        assert_eq!(count_msgs, 1);
    }
}
