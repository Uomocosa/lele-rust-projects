use super::error::Error;
use super::rollback_session::RollbackSession;
use super::simulation::Simulation;

pub fn predict<S>(session: &mut RollbackSession<S>, inputs: Vec<S::Input>) -> Result<u64, Error>
where
    S: Simulation,
    S::State: Clone,
{
    let committed = session.committed_tick;
    let next = session.predicted_tick + 1;
    if next.saturating_sub(committed) > session.config.prediction_lookahead {
        return Err(Error::PredictionLookaheadExceeded {
            limit: session.config.prediction_lookahead,
        });
    }
    session.sim.step(next, &inputs);
    session.pending.push(inputs);
    session.pending_hashes.push(session.sim.hash());
    session.predicted_tick = next;
    Ok(next)
}

#[cfg(test)]
mod tests {
    use super::super::rollback_config;
    use super::super::rollback_session::RollbackSession;
    use super::super::test_sim;
    use super::predict;

    #[test]
    fn test_usage() {
        let mut session = RollbackSession::new(
            test_sim::TestSim::default(),
            rollback_config::RollbackConfig::default(),
        );
        let tick = predict(&mut session, vec![1, 2]).unwrap();
        assert_eq!(tick, 1);
        assert!(session.predicted_tick() > session.committed_tick());
        let (position, _velocity) = session.predicted_state();
        assert!(position > 0);
    }

    #[test]
    fn prediction_respects_lookahead() {
        let config = rollback_config::RollbackConfig {
            prediction_lookahead: 2,
            max_committed_frames: 64,
        };
        let mut session = RollbackSession::new(test_sim::TestSim::default(), config);
        predict(&mut session, vec![1]).unwrap();
        predict(&mut session, vec![1]).unwrap();
        assert!(predict(&mut session, vec![1]).is_err());
    }
}
