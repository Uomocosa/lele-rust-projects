use crate::clicker;

pub fn increment(state: &mut clicker::State, amount: u64) {
    state.count = state.count.wrapping_add(amount);
    let cmd = clicker::Command::Increment { count: state.count };
    let _ = state.cmd_tx.send(cmd);
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use tokio::sync::mpsc;

    use super::increment;
    use crate::clicker;

    #[test]
    fn test_usage() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let mut state = clicker::State {
            event_rx: Mutex::new(mpsc::unbounded_channel().1),
            cmd_tx: tx,
            contract_key: None,
            count: 5,
            status: clicker::ConnectionStatus::Connecting,
            log: std::collections::VecDeque::new(),
        };

        increment(&mut state, 3);
        assert_eq!(state.count, 8);

        let cmd = rx.try_recv().unwrap();
        match cmd {
            clicker::Command::Increment { count } => assert_eq!(count, 8),
        }
    }
}
