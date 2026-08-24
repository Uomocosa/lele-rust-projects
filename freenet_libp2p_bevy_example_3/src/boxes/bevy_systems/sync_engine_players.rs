use bevy::prelude::*;

use crate::boxes;
use crate::engine;
use crate::roster;

/// Ensures every known player (self plus each roster peer) is spawned in the sim worker. Runs each
/// fixed tick so a peer joining the roster late immediately gains an engine body (spawns are queued
/// ahead of that tick's steps, which the worker processes in order).
pub fn sync_engine_players(
    engine: Res<engine::EngineHandle>,
    config: Res<boxes::Config>,
    roster: Res<roster::Roster>,
) {
    engine.send_cmd(engine::EngineCmd::Spawn(**config));
    for id in roster.keys() {
        engine.send_cmd(engine::EngineCmd::Spawn(*id));
    }
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use bevy::prelude::*;

    use super::sync_engine_players;
    use crate::boxes;
    use crate::engine;
    use crate::roster;

    #[test]
    fn test_usage() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let mut app = App::new();
        app.insert_resource(boxes::Config::new([1; 32]));
        let mut entries = roster::RosterState::default();
        entries.insert(
            [2; 32],
            roster::PeerEntry {
                peer_id: "peer".to_string(),
                addrs: vec![],
                seq: now,
                signature: Vec::new(),
            },
        );
        app.insert_resource(roster::Roster(entries));
        app.insert_resource(engine::spawn_engine());
        app.add_systems(Update, sync_engine_players);
        app.update();

        let handle = app.world().resource::<engine::EngineHandle>();
        handle.send_cmd(engine::EngineCmd::Spawn([1; 32]));
        handle.send_cmd(engine::EngineCmd::Spawn([2; 32]));
        handle.send_cmd(engine::EngineCmd::Step {
            tick: 1,
            actions: vec![
                ([1; 32], engine::Action::default()),
                ([2; 32], engine::Action::default()),
            ],
        });
        let reply = handle.recv_reply();
        assert!(matches!(
            reply,
            Some(engine::EngineReply::Snapshot(s)) if s.bodies.len() == 2
        ));
    }
}
