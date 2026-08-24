use super::error::Error;
use super::rollback_session::RollbackSession;
use super::simulation::Simulation;

pub fn authoritative_state<S>(session: &RollbackSession<S>) -> Result<S::State, Error>
where
    S: Simulation,
    S::State: Clone,
{
    session
        .committed_frames
        .last()
        .map(|frame| frame.state.clone())
        .ok_or(Error::NoCommittedFrame)
}

#[cfg(test)]
mod tests {
    use super::super::rollback_config;
    use super::super::rollback_session::RollbackSession;
    use super::super::test_sim;
    use super::authoritative_state;

    #[test]
    fn test_usage() {
        let mut session = RollbackSession::new(
            test_sim::TestSim::default(),
            rollback_config::RollbackConfig::default(),
        );
        session.predict(vec![1]).unwrap();
        session.commit(vec![1]).unwrap();
        let (position, _velocity) = authoritative_state(&session).unwrap();
        assert_eq!(position, 1);
    }
}
