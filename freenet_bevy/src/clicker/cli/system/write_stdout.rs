use bevy::prelude::*;

use crate::clicker::ClickerEvent;
use crate::clicker::resource::State::ClickerState;

pub fn write_stdout(state: ResMut<ClickerState>) {
    let mut event_rx = state.event_rx.lock().unwrap();
    while let Ok(event) = event_rx.try_recv() {
        match event {
            ClickerEvent::Init {
                contract_key,
                count,
            } => {
                println!("[event] init: contract={}, count={}", contract_key, count);
            }
            ClickerEvent::Notification { count } => {
                println!("[event] notification: count={}", count);
            }
            ClickerEvent::UpdateResponse { count } => {
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
