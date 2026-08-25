use crate::engine;
use crate::netcode;

pub fn sync_participants(lockstep: &mut netcode::Lockstep, ids: &[engine::PlayerId]) {
    let mut merged = lockstep.participants.clone();
    merged.extend_from_slice(ids);
    merged.sort_unstable();
    merged.dedup();
    lockstep.participants = merged;
}

#[cfg(test)]
mod tests {
    use crate::netcode;

    use super::sync_participants;

    #[test]
    fn test_usage() {
        let mut lockstep = netcode::Lockstep::new(vec![]);
        sync_participants(&mut lockstep, &[[2; 32], [1; 32], [2; 32]]);
        assert_eq!(lockstep.participants, vec![[1; 32], [2; 32]]);

        sync_participants(&mut lockstep, &[[3; 32], [1; 32]]);
        assert_eq!(lockstep.participants, vec![[1; 32], [2; 32], [3; 32]]);
    }
}
