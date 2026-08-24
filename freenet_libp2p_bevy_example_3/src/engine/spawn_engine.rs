use std::sync::mpsc;
use std::thread;

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
    let mut sim = engine::Engine::new();
    while let Ok(cmd) = cmd_rx.recv() {
        match cmd {
            engine::EngineCmd::Spawn(id) => sim.spawn_player(id),
            engine::EngineCmd::Step { tick, actions } => match sim.step(tick, &actions) {
                Ok(snapshot) => {
                    let _ = reply_tx.send(engine::EngineReply::Snapshot(snapshot));
                }
                Err(e) => {
                    tracing::warn!(target: "engine", tick, error = %e, "engine step failed");
                }
            },
        }
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
            tick: 2,
            actions: Vec::new(),
        });
        let snapshot = handle.recv_engine();
        assert_eq!(snapshot.map(|s| s.bodies.len()), Some(1));
    }
}
