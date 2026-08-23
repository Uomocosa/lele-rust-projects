use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::input_log_entry;
use crate::roster_state;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ContractState {
    pub roster: roster_state::RosterState,
    pub input_log: BTreeMap<[u8; 32], input_log_entry::InputLogEntry>,
}

#[cfg(test)]
mod tests {
    use super::ContractState;

    #[test]
    fn test_usage() {
        let state = ContractState::default();
        assert!(state.roster.is_empty());
        assert!(state.input_log.is_empty());
    }
}
