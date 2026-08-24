use super::error::Error;
use super::rollback_session::RollbackSession;
use super::simulation::Simulation;

pub fn authoritative_hash<S>(session: &RollbackSession<S>) -> Result<u64, Error>
where
    S: Simulation,
    S::State: Clone,
{
    session
        .committed_frames
        .last()
        .map(|frame| frame.hash)
        .ok_or(Error::NoCommittedFrame)
}

#[cfg(test)]
mod tests {
    use super::super::rollback_config;
    use super::super::rollback_session::RollbackSession;
    use super::super::test_sim;
    use super::authoritative_hash;

    #[test]
    fn test_usage() {
        let mut session = RollbackSession::new(
            test_sim::TestSim::default(),
            rollback_config::RollbackConfig::default(),
        );
        let before = authoritative_hash(&session).unwrap();
        session.commit(vec![1]).unwrap();
        assert_ne!(authoritative_hash(&session).unwrap(), before);
    }
}
