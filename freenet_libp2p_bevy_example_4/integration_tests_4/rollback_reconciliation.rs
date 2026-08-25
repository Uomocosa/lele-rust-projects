use bevy_lele_rollback_plugin_1::{RollbackConfig, RollbackSession};
use freenet_libp2p_bevy_example_4_lib::engine;

const LOCAL: engine::PlayerId = [1; 32];
const REMOTE: engine::PlayerId = [2; 32];

fn action(direction: engine::Direction) -> engine::Action {
    engine::Action {
        direction,
        jump: false,
    }
}

fn spawn_two() -> engine::Engine {
    let mut engine = engine::Engine::new();
    engine.spawn_player(LOCAL);
    engine.spawn_player(REMOTE);
    engine
}

fn authoritative_trace() -> Vec<Vec<(engine::PlayerId, engine::Action)>> {
    let this = action(engine::Direction::Right);
    let that = action(engine::Direction::Right);
    vec![
        vec![(LOCAL, this), (REMOTE, that)],
        vec![(LOCAL, this), (REMOTE, that)],
    ]
}

/// Mirrors the rollback crate's own reconciliation test against the real headless engine: a
/// guessed remote input that diverges from the authoritative input triggers a rollback on commit.
///
/// Avian's solver keeps per-body state beyond position+velocity, so a predicated session restores
/// losslessly only against a run with the *same* update shape. The ground truth is therefore a
/// same-shape session that only ever feeds the authoritative input set (a "direct authoritative
/// run" that never guesses wrong); the divergent session must roll back and converge onto exactly
/// that authoritative trajectory.
#[test]
fn rollback_reconciles_with_authoritative_trace() {
    let trace = authoritative_trace();
    let guessed_remote = action(engine::Direction::Left);

    let mut subject = RollbackSession::new(spawn_two(), RollbackConfig::default());
    subject
        .predict(vec![(REMOTE, guessed_remote)])
        .expect("predict within lookahead");
    subject
        .predict(vec![(REMOTE, guessed_remote)])
        .expect("predict within lookahead");

    let first = subject
        .commit(trace[0].clone())
        .expect("commit should succeed");
    assert!(
        first.diverged,
        "the guessed remote input must have diverged from the authoritative input"
    );
    let second = subject
        .commit(trace[1].clone())
        .expect("commit should succeed");
    assert!(
        second.diverged,
        "the rerun guess must keep diverging until the authoritative trace catches up"
    );

    let mut reference = RollbackSession::new(spawn_two(), RollbackConfig::default());
    reference
        .predict(trace[0].clone())
        .expect("predict within lookahead");
    reference
        .predict(trace[1].clone())
        .expect("predict within lookahead");
    let _ = reference.commit(trace[0].clone()).unwrap();
    let _ = reference.commit(trace[1].clone()).unwrap();

    assert_eq!(
        subject
            .authoritative_hash()
            .expect("subject authoritative hash"),
        reference
            .authoritative_hash()
            .expect("direct authoritative run hash"),
        "after rollback the divergent session must converge onto a direct authoritative run"
    );
}
