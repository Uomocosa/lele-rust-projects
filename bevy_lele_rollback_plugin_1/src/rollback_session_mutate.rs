use super::error::Error;
use super::rollback_session::RollbackSession;
use super::simulation::Simulation;

/// Gives the caller mutable access to the underlying simulation so a participant joining late can
/// be registered after the session has already been initialised (e.g. `engine.spawn_player`).
pub fn mutate<S>(session: &mut RollbackSession<S>, f: impl FnOnce(&mut S)) -> Result<(), Error>
where
    S: Simulation,
    S::State: Clone,
{
    f(&mut session.sim);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::rollback_config;
    use super::super::simulation::Simulation;
    use super::super::test_sim;
    use super::mutate;
    use crate::RollbackSession;

    #[test]
    fn test_usage() {
        let mut session = RollbackSession::new(
            test_sim::TestSim::default(),
            rollback_config::RollbackConfig::default(),
        );
        mutate(&mut session, |sim| Simulation::step(sim, 1, &[3])).unwrap();
        let (position, _velocity) = session.predicted_state();
        assert_eq!(position, 3);
    }
}
