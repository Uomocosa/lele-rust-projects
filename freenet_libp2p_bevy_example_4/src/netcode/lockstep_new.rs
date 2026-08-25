use crate::engine;
use crate::netcode;

pub fn new(mut participants: Vec<engine::PlayerId>) -> netcode::Lockstep {
    participants.sort_unstable();
    participants.dedup();
    netcode::Lockstep {
        participants,
        commits: Default::default(),
        reveals: Default::default(),
        tampered: Default::default(),
        late_streak: Default::default(),
        offline: Vec::new(),
        applied_through: 0,
        tick_plans: Default::default(),
    }
}

#[cfg(test)]
mod tests {
    use crate::netcode;

    #[test]
    fn test_usage() {
        let lockstep = netcode::Lockstep::new(vec![[2; 32], [1; 32], [2; 32]]);
        assert_eq!(lockstep.participants, vec![[1; 32], [2; 32]]);
        assert_eq!(lockstep.applied_through, 0);
    }
}
