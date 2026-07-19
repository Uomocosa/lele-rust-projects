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
