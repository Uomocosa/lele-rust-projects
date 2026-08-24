use super::rollback_session::RollbackSession;
use super::simulation::Simulation;

pub fn predicted_state<S>(session: &RollbackSession<S>) -> S::State
where
    S: Simulation,
    S::State: Clone,
{
    session.sim.snapshot()
}

#[cfg(test)]
mod tests {
    use super::super::rollback_config;
    use super::super::rollback_session::RollbackSession;
    use super::super::test_sim;
    use super::predicted_state;

    #[test]
    fn test_usage() {
        let mut session = RollbackSession::new(
            test_sim::TestSim::default(),
            rollback_config::RollbackConfig::default(),
        );
        session.predict(vec![2]).unwrap();
        let (position, velocity) = predicted_state(&session);
        assert_eq!((position, velocity), (2, 2));
    }
}
