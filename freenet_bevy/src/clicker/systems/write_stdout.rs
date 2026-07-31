use bevy::prelude::*;

use crate::clicker::event::Event;
use crate::clicker::state::State;

pub fn write_stdout(state: ResMut<State>) {
    let mut event_rx = state.event_rx.lock().unwrap();
    while let Ok(event) = event_rx.try_recv() {
        match event {
            Event::Init {
                contract_key,
                count,
            } => {
                println!("[event] init: contract={}, count={}", contract_key, count);
            }
            Event::Notification { count } => {
                println!("[event] notification: count={}", count);
            }
            Event::UpdateResponse { count } => {
                println!("[event] update response: count={}", count);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::write_stdout;

    #[test]
    fn test_usage() {
        let _ = write_stdout;
    }
}
