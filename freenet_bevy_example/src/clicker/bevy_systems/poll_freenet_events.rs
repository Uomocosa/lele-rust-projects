use bevy::prelude::*;

use crate::clicker;
use crate::clicker::state::LOG_CAPACITY;

fn push_log(state: &mut clicker::State, text: impl Into<String>) -> clicker::LogMessage {
    let msg = clicker::LogMessage::new(text);
    state.log.push_back(msg.clone());
    while state.log.len() > LOG_CAPACITY {
        state.log.pop_front();
    }
    msg
}

pub fn poll_freenet_events(
    state: ResMut<clicker::State>,
    mut count_writer: MessageWriter<clicker::CountChanged>,
    mut connection_writer: MessageWriter<clicker::ConnectionChanged>,
    mut log_writer: MessageWriter<clicker::LogMessageAdded>,
) {
    let mut rx = state.event_rx.lock().unwrap();
    while let Ok(event) = rx.try_recv() {
        match event {
            clicker::Event::Notification { count } => {
                drop(rx);
                let mut s = state;
                s.count = count;
                let msg = push_log(&mut s, format!("counter updated to {count}"));
                count_writer.write(clicker::CountChanged { count });
                log_writer.write(clicker::LogMessageAdded(msg));
                return;
            }
            clicker::Event::UpdateResponse { count } => {
                drop(rx);
                let mut s = state;
                s.count = count;
                let msg = push_log(&mut s, format!("counter updated to {count}"));
                count_writer.write(clicker::CountChanged { count });
                log_writer.write(clicker::LogMessageAdded(msg));
                return;
            }
            clicker::Event::Init {
                contract_key,
                count,
            } => {
                drop(rx);
                let mut s = state;
                s.status = clicker::ConnectionStatus::Connected;
                s.contract_key = Some(contract_key);
                s.count = count;
                let connected_msg = push_log(&mut s, "connected to freenet");
                let count_msg = push_log(&mut s, format!("got counter state ({count})"));
                connection_writer.write(clicker::ConnectionChanged {
                    status: clicker::ConnectionStatus::Connected,
                });
                count_writer.write(clicker::CountChanged { count });
                log_writer.write(clicker::LogMessageAdded(connected_msg));
                log_writer.write(clicker::LogMessageAdded(count_msg));
                return;
            }
            clicker::Event::ConnectionError(reason) => {
                drop(rx);
                let mut s = state;
                s.status = clicker::ConnectionStatus::Error(reason.clone());
                let msg = push_log(&mut s, format!("could not connect: {reason}"));
                connection_writer.write(clicker::ConnectionChanged {
                    status: clicker::ConnectionStatus::Error(reason),
                });
                log_writer.write(clicker::LogMessageAdded(msg));
                return;
            }
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
        app.add_message::<clicker::ConnectionChanged>();
        app.add_message::<clicker::LogMessageAdded>();

        let (evt_tx, evt_rx) = mpsc::unbounded_channel();
        let (cmd_tx, _cmd_rx) = mpsc::unbounded_channel();
        app.insert_resource(clicker::State {
            event_rx: std::sync::Mutex::new(evt_rx),
            cmd_tx,
            contract_key: None,
            count: 0,
            status: clicker::ConnectionStatus::Connecting,
            log: std::collections::VecDeque::new(),
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
        assert_eq!(state.log.len(), 1);

        let count_msgs = app
            .world()
            .resource::<Messages<clicker::CountChanged>>()
            .len();
        assert_eq!(count_msgs, 1);
    }
}
