use super::commit_report::CommitReport;
use super::committed_frame::CommittedFrame;
use super::error::Error;
use super::rollback_session::RollbackSession;
use super::simulation::Simulation;

pub fn commit<S>(
    session: &mut RollbackSession<S>,
    inputs: Vec<S::Input>,
) -> Result<CommitReport, Error>
where
    S: Simulation,
    S::State: Clone,
{
    let next = session.committed_tick + 1;
    if session.predicted_tick >= next {
        let base = session
            .committed_frames
            .last()
            .ok_or(Error::NoCommittedFrame)?;
        session.sim.restore(base.state.clone());
        session.sim.step(next, &inputs);
        let authoritative_hash = session.sim.hash();
        let state = session.sim.snapshot();
        let diverged = matches!(
            session.pending_hashes.first(),
            Some(h) if *h != authoritative_hash
        );
        push_committed(
            session,
            CommittedFrame {
                tick: next,
                state,
                hash: authoritative_hash,
            },
        );
        session.committed_tick = next;
        session.pending.remove(0);
        session.pending_hashes.remove(0);
        let mut tick = next;
        let mut rederived = Vec::with_capacity(session.pending.len());
        for batch in &session.pending {
            tick += 1;
            session.sim.step(tick, batch);
            rederived.push(session.sim.hash());
        }
        session.pending_hashes = rederived;
        session.predicted_tick = tick;
        Ok(CommitReport {
            tick: next,
            diverged,
            authoritative_hash,
        })
    } else {
        session.sim.step(next, &inputs);
        let authoritative_hash = session.sim.hash();
        let state = session.sim.snapshot();
        push_committed(
            session,
            CommittedFrame {
                tick: next,
                state,
                hash: authoritative_hash,
            },
        );
        session.committed_tick = next;
        session.predicted_tick = next;
        Ok(CommitReport {
            tick: next,
            diverged: false,
            authoritative_hash,
        })
    }
}

// needed helper: append a committed frame, evicting the oldest beyond the configured cap
fn push_committed<S: Simulation>(session: &mut RollbackSession<S>, frame: CommittedFrame<S::State>)
where
    S::State: Clone,
{
    session.committed_frames.push(frame);
    if session.committed_frames.len() > session.config.max_committed_frames {
        session.committed_frames.remove(0);
    }
}

#[cfg(test)]
mod tests {
    use super::super::rollback_config;
    use super::super::rollback_session::RollbackSession;
    use super::super::simulation::Simulation;
    use super::super::test_sim;
    use super::commit;

    fn run_authoritative(trace: &[Vec<i64>]) -> u64 {
        let mut sim = test_sim::TestSim::default();
        for inputs in trace {
            sim.step(0, inputs);
        }
        sim.hash()
    }

    #[test]
    fn test_usage() {
        let mut a = RollbackSession::new(
            test_sim::TestSim::default(),
            rollback_config::RollbackConfig::default(),
        );
        let mut b = RollbackSession::new(
            test_sim::TestSim::default(),
            rollback_config::RollbackConfig::default(),
        );
        let trace = [vec![1], vec![1, 1], vec![2]];
        for inputs in &trace {
            commit(&mut a, inputs.clone()).unwrap();
            commit(&mut b, inputs.clone()).unwrap();
        }
        assert_eq!(
            a.authoritative_hash().unwrap(),
            b.authoritative_hash().unwrap()
        );
    }

    #[test]
    fn rollback_reconciles_with_authoritative_trace() {
        let mut session = RollbackSession::new(
            test_sim::TestSim::default(),
            rollback_config::RollbackConfig::default(),
        );
        session.predict(vec![2]).unwrap();
        session.predict(vec![0]).unwrap();
        let first = commit(&mut session, vec![1]).unwrap();
        assert!(first.diverged);
        let second = commit(&mut session, vec![1]).unwrap();
        assert!(second.diverged);
        let expected = run_authoritative(&[vec![1], vec![1]]);
        assert_eq!(session.authoritative_hash().unwrap(), expected);
    }

    #[test]
    fn prediction_without_remote_guess_does_not_diverge() {
        let mut session = RollbackSession::new(
            test_sim::TestSim::default(),
            rollback_config::RollbackConfig::default(),
        );
        session.predict(vec![1]).unwrap();
        let report = commit(&mut session, vec![1]).unwrap();
        assert!(!report.diverged);
    }
}
