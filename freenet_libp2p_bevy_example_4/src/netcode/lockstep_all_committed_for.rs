use crate::netcode;

/// True when every non-offline synced participant has recorded a commit for `tick`. A peer may
/// only reveal (and have its reveal accepted) once this holds, guaranteeing the hash-first,
/// reveal-last commitment ordering.
pub fn all_committed_for(lockstep: &netcode::Lockstep, tick: u64) -> bool {
    lockstep.participants.iter().all(|peer| {
        lockstep.offline.contains(peer) || lockstep.commits.contains_key(&(tick, *peer))
    })
}

#[cfg(test)]
mod tests {
    use crate::engine;
    use crate::netcode;

    use super::all_committed_for;

    #[test]
    fn test_usage() {
        let mut lockstep = netcode::Lockstep::new(vec![[1; 32], [2; 32]]);
        assert!(!all_committed_for(&lockstep, 3));
        lockstep.record_commit(3, [1; 32], 1).unwrap();
        assert!(!all_committed_for(&lockstep, 3));
        lockstep.record_commit(3, [2; 32], 2).unwrap();
        assert!(all_committed_for(&lockstep, 3));
    }

    #[test]
    fn offline_peers_do_not_block_reveal() {
        let mut lockstep = netcode::Lockstep::new(vec![[1; 32]]);
        lockstep.record_commit(5, [1; 32], 42).unwrap();
        assert!(all_committed_for(&lockstep, 5));
        let _ = engine::Action::default();
    }
}
