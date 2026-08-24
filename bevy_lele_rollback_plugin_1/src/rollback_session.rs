use super::commit_report::CommitReport;
use super::committed_frame::CommittedFrame;
use super::error::Error;
use super::rollback_config::RollbackConfig;
use super::rollback_session_authoritative_hash;
use super::rollback_session_authoritative_state;
use super::rollback_session_commit;
use super::rollback_session_committed_tick;
use super::rollback_session_mutate;
use super::rollback_session_new;
use super::rollback_session_predict;
use super::rollback_session_predicted_state;
use super::rollback_session_predicted_tick;
use super::simulation::Simulation;

pub struct RollbackSession<S: Simulation>
where
    S::State: Clone,
{
    pub(crate) sim: S,
    pub(crate) committed_frames: Vec<CommittedFrame<S::State>>,
    pub(crate) committed_tick: u64,
    pub(crate) predicted_tick: u64,
    pub(crate) pending: Vec<Vec<S::Input>>,
    pub(crate) pending_hashes: Vec<u64>,
    pub(crate) config: RollbackConfig,
}

#[rustfmt::skip]
impl<S: Simulation> RollbackSession<S> where S::State: Clone {
    pub fn new(sim: S, config: RollbackConfig) -> Self { rollback_session_new::new(sim, config) }
    pub fn mutate(&mut self, f: impl FnOnce(&mut S)) -> Result<(), Error> { rollback_session_mutate::mutate(self, f) }
    pub fn predict(&mut self, inputs: Vec<S::Input>) -> Result<u64, Error> { rollback_session_predict::predict(self, inputs) }
    pub fn commit(&mut self, inputs: Vec<S::Input>) -> Result<CommitReport, Error> { rollback_session_commit::commit(self, inputs) }
    pub fn authoritative_state(&self) -> Result<S::State, Error> { rollback_session_authoritative_state::authoritative_state(self) }
    pub fn predicted_state(&self) -> S::State { rollback_session_predicted_state::predicted_state(self) }
    pub fn authoritative_hash(&self) -> Result<u64, Error> { rollback_session_authoritative_hash::authoritative_hash(self) }
    pub fn committed_tick(&self) -> u64 { rollback_session_committed_tick::committed_tick(self) }
    pub fn predicted_tick(&self) -> u64 { rollback_session_predicted_tick::predicted_tick(self) }
}

#[cfg(test)]
mod tests {
    use super::super::simulation::Simulation;
    use super::super::test_sim;
    use super::RollbackSession;
    use crate::rollback_config;

    #[test]
    fn test_usage() {
        let session = RollbackSession::new(
            test_sim::TestSim::default(),
            rollback_config::RollbackConfig::default(),
        );
        assert_eq!(session.committed_tick(), 0);
        assert_eq!(session.predicted_tick(), 0);
        assert_eq!(
            session.authoritative_hash().unwrap(),
            test_sim::TestSim::default().hash()
        );
    }
}
