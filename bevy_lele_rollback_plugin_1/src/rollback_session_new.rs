use super::committed_frame::CommittedFrame;
use super::constants::INITIAL_TICK;
use super::rollback_config::RollbackConfig;
use super::rollback_session::RollbackSession;
use super::simulation::Simulation;

pub fn new<S: Simulation>(sim: S, config: RollbackConfig) -> RollbackSession<S>
where
    S::State: Clone,
{
    let hash = sim.hash();
    let state = sim.snapshot();
    let committed_frames = vec![CommittedFrame {
        tick: INITIAL_TICK,
        state,
        hash,
    }];
    RollbackSession {
        sim,
        committed_frames,
        committed_tick: INITIAL_TICK,
        predicted_tick: INITIAL_TICK,
        pending: Vec::new(),
        pending_hashes: Vec::new(),
        config,
    }
}

#[cfg(test)]
mod tests {
    use super::super::rollback_config;
    use super::super::test_sim;
    use super::new;

    #[test]
    fn test_usage() {
        let session = new(
            test_sim::TestSim::default(),
            rollback_config::RollbackConfig::default(),
        );
        assert_eq!(session.committed_frames.len(), 1);
        assert_eq!(session.committed_frames[0].tick, 0);
        assert_eq!(session.pending.len(), 0);
    }
}
