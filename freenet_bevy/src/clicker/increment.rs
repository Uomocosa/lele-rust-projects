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
        let key = freenet_stdlib::prelude::ContractKey::from_params_and_code(
            freenet_stdlib::prelude::Parameters::from(Vec::new()),
            freenet_stdlib::prelude::ContractCode::from(Vec::new()),
        );
        let mut state = clicker::State {
            event_rx: Mutex::new(mpsc::unbounded_channel().1),
            cmd_tx: tx,
            contract_key: key,
            count: 5,
        };

        increment(&mut state, 3);
        assert_eq!(state.count, 8);

        let cmd = rx.try_recv().unwrap();
        match cmd {
            clicker::Command::Increment { count } => assert_eq!(count, 8),
        }
    }
}
