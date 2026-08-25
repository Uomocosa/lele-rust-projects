use crate::engine;
use crate::netcode;

/// Records a revealed input, hash-checked against the peer's earlier commit. Commitment-first
/// ordering (a peer only *sends* its reveal once every synced participant has committed) is
/// enforced on the sending side via `Lockstep::all_committed_for`; here we accept any reveal so an
/// out-of-order reveal arriving ahead of its commit is never dropped.
pub fn record_reveal(
    lockstep: &mut netcode::Lockstep,
    tick: u64,
    peer: engine::PlayerId,
    action: engine::Action,
) -> Result<(), netcode::Error> {
    let key = (tick, peer);
    if let Some(&committed) = lockstep.commits.get(&key)
        && engine::hash_action(&action) != committed
    {
        lockstep.tampered.insert(peer);
        return Err(netcode::Error::RevealMismatch);
    }
    lockstep.reveals.insert(key, action);
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::engine;
    use crate::netcode;

    #[test]
    fn test_usage() {
        let mut lockstep = netcode::Lockstep::new(vec![[1; 32]]);
        let action = engine::Action {
            direction: engine::Direction::Right,
            jump: false,
        };
        lockstep
            .record_commit(4, [1; 32], engine::hash_action(&action))
            .unwrap();
        assert!(lockstep.record_reveal(4, [1; 32], action).is_ok());
    }

    #[test]
    fn tampered_reveal_is_flagged() {
        let mut lockstep = netcode::Lockstep::new(vec![[1; 32]]);
        let committed = engine::Action {
            direction: engine::Direction::Right,
            jump: false,
        };
        lockstep
            .record_commit(9, [1; 32], engine::hash_action(&committed))
            .unwrap();

        let tampered = engine::Action {
            direction: engine::Direction::Left,
            jump: true,
        };
        assert_eq!(
            lockstep.record_reveal(9, [1; 32], tampered),
            Err(netcode::Error::RevealMismatch)
        );
        assert!(lockstep.tampered.contains(&[1; 32]));
        assert!(!lockstep.reveals.contains_key(&(9, [1; 32])));
    }

    #[test]
    fn reveal_without_prior_known_commit_is_accepted() {
        let mut lockstep = netcode::Lockstep::new(vec![[1; 32], [2; 32]]);
        let action = engine::Action::default();
        assert!(lockstep.record_reveal(3, [1; 32], action).is_ok());
        lockstep
            .record_commit(3, [1; 32], engine::hash_action(&action))
            .unwrap();
        let tampered = engine::Action {
            direction: engine::Direction::Left,
            jump: true,
        };
        assert_eq!(
            lockstep.record_reveal(3, [1; 32], tampered),
            Err(netcode::Error::RevealMismatch)
        );
    }
}
