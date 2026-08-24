use bevy::prelude::*;

use crate::boxes;
use crate::engine;
use crate::p2p;
use crate::roster;

/// The per-tick lockstep pipeline: capture local input -> commit + reveal over libp2p -> drain
/// peers' commits/reveals/state-hashes into the `Lockstep` -> `advance_to` and step the
/// deterministic sim worker -> broadcast the resulting state hash and compare against peers.
#[allow(clippy::too_many_arguments)]
pub fn netcode_tick(
    commands: Res<p2p::P2pCommands>,
    mut events: ResMut<p2p::P2pEvents>,
    engine: Res<engine::EngineHandle>,
    mut lockstep: ResMut<boxes::NetcodeLockstep>,
    mut state: ResMut<boxes::SimState>,
    mut snapshot: ResMut<boxes::LatestSnapshot>,
    keyboard: Res<ButtonInput<KeyCode>>,
    config: Res<boxes::Config>,
    roster: Res<roster::Roster>,
) {
    state.clock += 1;
    let now = state.clock;
    let own = **config;

    lockstep.sync_participants(&roster.keys().copied().collect::<Vec<_>>());

    let action = read_action(&keyboard);
    let hash = engine::hash_action(&action);
    let _ = lockstep.record_commit(now, own, hash);
    let _ = lockstep.record_reveal(now, own, action);

    for (id, entry) in roster.iter() {
        if *id == own {
            continue;
        }
        send(
            &commands,
            &entry.peer_id,
            p2p::NetcodeMsg::Commit {
                tick: now,
                player_id: own,
                hash,
            },
        );
        send(
            &commands,
            &entry.peer_id,
            p2p::NetcodeMsg::Reveal {
                tick: now,
                player_id: own,
                action,
            },
        );
    }

    while let Ok(event) = events.try_recv() {
        let p2p::Event::IncomingNetcode { from, msg } = event else {
            continue;
        };
        let sender = player_id_of(&roster, &from);
        match msg {
            p2p::NetcodeMsg::Commit {
                tick,
                player_id,
                hash,
            } => {
                tracing::debug!(
                    target: "p2p",
                    tick,
                    player_id = %hex::encode(player_id),
                    "received netcode commit"
                );
                let _ = lockstep.record_commit(tick, player_id, hash);
            }
            p2p::NetcodeMsg::Reveal {
                tick,
                player_id,
                action,
            } => {
                tracing::debug!(
                    target: "p2p",
                    tick,
                    player_id = %hex::encode(player_id),
                    "received peer input"
                );
                let _ = lockstep.record_reveal(tick, player_id, action);
                if player_id != own {
                    state.seen_peers.insert(player_id);
                }
            }
            p2p::NetcodeMsg::StateHash { tick, hash } => {
                if let Some(pid) = sender {
                    state.peer_hashes.entry(pid).or_default().insert(tick, hash);
                }
            }
        }
    }

    for plan in lockstep.advance_to(now) {
        engine.send_cmd(engine::EngineCmd::Step {
            tick: plan.tick,
            actions: plan.ordered_inputs.clone(),
        });
        let Some(snapshot_value) = engine.recv_engine() else {
            continue;
        };
        **snapshot = Some(snapshot_value.clone());
        let h = engine::hash_snapshot(&snapshot_value);
        state.latest_hash = Some(h);
        for (id, entry) in roster.iter() {
            if *id == own {
                continue;
            }
            send(
                &commands,
                &entry.peer_id,
                p2p::NetcodeMsg::StateHash {
                    tick: plan.tick,
                    hash: h,
                },
            );
        }
        for pid in snapshot_value.bodies.keys() {
            if let Some(theirs) = state.peer_hashes.get(pid).and_then(|m| m.get(&plan.tick))
                && *theirs != h
            {
                tracing::warn!(
                    target: "p2p",
                    peer = %hex::encode(pid),
                    tick = plan.tick,
                    mine = h,
                    theirs = *theirs,
                    "state hash divergence"
                );
            }
        }
    }
}

// needed helper:
fn read_action(keyboard: &ButtonInput<KeyCode>) -> engine::Action {
    let left = keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft);
    let right = keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight);
    let direction = match (left, right) {
        (true, false) => engine::Direction::Left,
        (false, true) => engine::Direction::Right,
        _ => engine::Direction::Center,
    };
    let jump = keyboard.just_pressed(KeyCode::Space);
    engine::Action { direction, jump }
}

// needed helper:
fn send(commands: &p2p::P2pCommands, peer_id: &str, msg: p2p::NetcodeMsg) {
    let _ = commands.send(p2p::Command::SendNetcode {
        peer_id: peer_id.to_string(),
        msg,
    });
}

// needed helper:
fn player_id_of(roster: &roster::Roster, from: &libp2p::PeerId) -> Option<engine::PlayerId> {
    let base = from.to_base58();
    roster
        .iter()
        .find_map(|(id, entry)| (entry.peer_id == base).then_some(*id))
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use bevy::prelude::*;

    use super::netcode_tick;
    use crate::boxes;
    use crate::engine;
    use crate::netcode;
    use crate::p2p;
    use crate::roster;

    #[test]
    fn test_usage() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let (cmd_tx, _cmd_rx) = tokio::sync::mpsc::unbounded_channel::<p2p::Command>();
        let (_event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel::<p2p::Event>();
        let mut app = App::new();
        app.insert_resource(p2p::P2pCommands(cmd_tx));
        app.insert_resource(p2p::P2pEvents(event_rx));
        app.insert_resource(boxes::Config::new([1; 32]));
        app.insert_resource(engine::spawn_engine());
        app.insert_resource(boxes::LatestSnapshot::default());
        app.insert_resource(boxes::NetcodeLockstep(netcode::Lockstep::new(vec![])));
        app.insert_resource(boxes::SimState::default());
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
        app.insert_resource(ButtonInput::<KeyCode>::default());
        app.add_systems(Update, netcode_tick);
        for _ in 0..30 {
            app.update();
        }

        let clock = app.world().resource::<boxes::SimState>().clock;
        assert!(clock >= 30);
        let latest = app.world().resource::<boxes::SimState>().latest_hash;
        assert!(latest.is_some());
        let snap = app.world().resource::<boxes::LatestSnapshot>().0.is_some();
        assert!(snap);
    }
}
