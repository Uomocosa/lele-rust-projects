use crate::engine;
use crate::netcode;

pub fn record_commit(
    lockstep: &mut netcode::Lockstep,
    tick: u64,
    peer: engine::PlayerId,
    hash: u64,
) -> Result<(), netcode::Error> {
    let key = (tick, peer);
    if lockstep.commits.contains_key(&key) {
        return Err(netcode::Error::DuplicateCommit);
    }
    lockstep.commits.insert(key, hash);
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::netcode;

    #[test]
    fn test_usage() {
        let mut lockstep = netcode::Lockstep::new(vec![[1; 32]]);
        lockstep.record_commit(3, [1; 32], 42).unwrap();
        assert_eq!(lockstep.commits.get(&(3, [1; 32])), Some(&42));
        assert!(lockstep.record_commit(3, [1; 32], 43).is_err());
    }
}
