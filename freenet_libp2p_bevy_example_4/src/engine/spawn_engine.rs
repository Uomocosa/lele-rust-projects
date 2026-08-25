use std::collections::BTreeSet;
use std::sync::mpsc;
use std::thread;

use bevy_lele_rollback_plugin_1::{RollbackConfig, RollbackSession};

use crate::engine;

pub fn spawn_engine() -> engine::EngineHandle {
    let (cmd_tx, cmd_rx) = mpsc::channel::<engine::EngineCmd>();
    let (reply_tx, reply_rx) = mpsc::channel::<engine::EngineReply>();

    let _ = thread::Builder::new()
        .name("sim-engine".to_string())
        .spawn(move || run_worker(cmd_rx, reply_tx));

    engine::EngineHandle {
        cmd: cmd_tx,
        reply: std::sync::Mutex::new(reply_rx),
    }
}

// needed helper:
fn run_worker(
    cmd_rx: mpsc::Receiver<engine::EngineCmd>,
    reply_tx: mpsc::Sender<engine::EngineReply>,
) {
    let mut auth = engine::Engine::new();
    let mut spawned: BTreeSet<engine::PlayerId> = BTreeSet::new();
    let mut session: Option<RollbackSession<engine::Engine>> = None;
    while let Ok(cmd) = cmd_rx.recv() {
        match cmd {
            engine::EngineCmd::Spawn(id) => {
                auth.spawn_player(id);
                if spawned.insert(id)
                    && let Some(s) = session.as_mut()
                {
                    let _ = s.mutate(|engine| engine.spawn_player(id));
                }
            }
            engine::EngineCmd::Step { tick, actions } => {
                match auth.step(tick, &actions) {
                    Ok(snapshot) => {
                        let _ = reply_tx.send(engine::EngineReply::Snapshot(snapshot));
                    }
                    Err(e) => {
                        tracing::warn!(target: "engine", tick, error = %e, "authoritative step failed");
                    }
                }
                if let Some(s) = session.as_mut() {
                    let _ = s.commit(actions);
                }
            }
            engine::EngineCmd::Predict { inputs } => {
                let s = session.get_or_insert_with(|| {
                    let mut sim = engine::Engine::new();
                    for id in &spawned {
                        sim.spawn_player(*id);
                    }
                    RollbackSession::new(sim, RollbackConfig::default())
                });
                let _ = s.predict(inputs);
                let _ = reply_tx.send(engine::EngineReply::Predicted(to_snapshot(
                    &s.predicted_state(),
                )));
            }
            engine::EngineCmd::GetSimState => {
                let state = auth.sim_state();
                let _ = reply_tx.send(engine::EngineReply::SimState(state));
            }
            engine::EngineCmd::Restore(state) => {
                auth.restore(&state);
                if let Some(s) = session.as_mut() {
                    let _ = s.mutate(|engine| engine.restore(&state));
                }
            }
        }
    }
}

// needed helper:
fn to_snapshot(state: &engine::EngineSimState) -> engine::Snapshot {
    let bodies = state
        .bodies
        .iter()
        .map(|(pid, body)| (*pid, (body.0, body.1)))
        .collect();
    engine::Snapshot {
        tick: state.tick,
        bodies,
    }
}

#[cfg(test)]
mod tests {
    use crate::engine;

    use super::spawn_engine;

    #[test]
    fn test_usage() {
        let handle = spawn_engine();
        handle.send_cmd(engine::EngineCmd::Spawn([2; 32]));
        handle.send_cmd(engine::EngineCmd::Step {
            tick: 1,
            actions: vec![([2; 32], engine::Action::default())],
        });
        let reply = handle.recv_reply();
        assert!(matches!(reply, Some(engine::EngineReply::Snapshot(s)) if s.bodies.len() == 1));
    }
}
