use super::rollback_session::RollbackSession;
use super::simulation::Simulation;

pub fn committed_tick<S>(session: &RollbackSession<S>) -> u64
where
    S: Simulation,
    S::State: Clone,
{
    session.committed_tick
}

#[cfg(test)]
mod tests {
    use super::super::rollback_config;
    use super::super::rollback_session::RollbackSession;
    use super::super::test_sim;
    use super::committed_tick;

    #[test]
    fn test_usage() {
        let mut session = RollbackSession::new(
            test_sim::TestSim::default(),
            rollback_config::RollbackConfig::default(),
        );
        session.commit(vec![1]).unwrap();
        assert_eq!(committed_tick(&session), 1);
    }
}
