use bevy::prelude::*;

use crate::boxes;
use crate::engine;
use crate::p2p;
use crate::roster;

/// The per-tick lockstep pipeline: capture local input -> commit its hash -> wait until every
/// synced participant has committed -> reveal the input -> drain peers' reveals -> `advance_to` and
/// step the plain authoritative engine (driving remote rendering + the convergence hash) -> predict
/// the local box through the rollback session for immediate rendering -> broadcast the state hash
/// and compare against peers.
#[allow(clippy::too_many_arguments)]
pub fn netcode_tick(
    commands: Res<p2p::P2pCommands>,
    mut events: ResMut<p2p::P2pEvents>,
    engine: Res<engine::EngineHandle>,
    mut lockstep: ResMut<boxes::NetcodeLockstep>,
    mut state: ResMut<boxes::SimState>,
    mut snapshot: ResMut<boxes::LatestSnapshot>,
    mut predicted: ResMut<boxes::PredictedSnapshot>,
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
    broadcast_commit(&commands, &roster, own, now, hash);
    state.pending_reveals.insert(now, action);

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

    let ready: Vec<u64> = state
        .pending_reveals
        .keys()
        .copied()
        .filter(|&t| lockstep.all_committed_for(t))
        .collect();
    for t in ready {
        if let Some(&a) = state.pending_reveals.get(&t) {
            let _ = lockstep.record_reveal(t, own, a);
            broadcast_reveal(&commands, &roster, own, t, a);
            state.pending_reveals.remove(&t);
        }
    }

    let mut steps = 0;
    for plan in lockstep.advance_to(now) {
        engine.send_cmd(engine::EngineCmd::Step {
            tick: plan.tick,
            actions: plan.ordered_inputs.clone(),
        });
        steps += 1;
    }
    engine.send_cmd(engine::EngineCmd::Predict {
        inputs: vec![(own, action)],
    });

    let mut last_auth: Option<engine::Snapshot> = None;
    let mut predicted_snap: Option<engine::Snapshot> = None;
    for _ in 0..=steps {
        if let Some(reply) = engine.recv_reply() {
            match reply {
                engine::EngineReply::Snapshot(snapshot_value) => last_auth = Some(snapshot_value),
                engine::EngineReply::Predicted(snapshot_value) => {
                    predicted_snap = Some(snapshot_value)
                }
            }
        }
    }

    if let Some(auth) = last_auth {
        **snapshot = Some(auth.clone());
        let h = engine::hash_snapshot(&auth);
        state.latest_hash = Some(h);
        for (id, entry) in roster.iter() {
            if *id == own {
                continue;
            }
            send(
                &commands,
                &entry.peer_id,
                p2p::NetcodeMsg::StateHash {
                    tick: auth.tick,
                    hash: h,
                },
            );
        }
        for pid in auth.bodies.keys() {
            if let Some(theirs) = state.peer_hashes.get(pid).and_then(|m| m.get(&auth.tick))
                && *theirs != h
            {
                tracing::warn!(
                    target: "p2p",
                    peer = %hex::encode(pid),
                    tick = auth.tick,
                    mine = h,
                    theirs = *theirs,
                    "state hash divergence"
                );
            }
        }
    }
    if let Some(snap) = predicted_snap {
        **predicted = Some(snap);
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
fn broadcast_commit(
    commands: &p2p::P2pCommands,
    roster: &roster::Roster,
    own: engine::PlayerId,
    now: u64,
    hash: u64,
) {
    for (id, entry) in roster.iter() {
        if *id == own {
            continue;
        }
        send(
            commands,
            &entry.peer_id,
            p2p::NetcodeMsg::Commit {
                tick: now,
                player_id: own,
                hash,
            },
        );
    }
}

// needed helper:
fn broadcast_reveal(
    commands: &p2p::P2pCommands,
    roster: &roster::Roster,
    own: engine::PlayerId,
    now: u64,
    action: engine::Action,
) {
    for (id, entry) in roster.iter() {
        if *id == own {
            continue;
        }
        send(
            commands,
            &entry.peer_id,
            p2p::NetcodeMsg::Reveal {
                tick: now,
                player_id: own,
                action,
            },
        );
    }
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
        let (event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel::<p2p::Event>();
        let mut app = App::new();
        app.insert_resource(p2p::P2pCommands(cmd_tx));
        app.insert_resource(p2p::P2pEvents(event_rx));
        app.insert_resource(boxes::Config::new([1; 32]));
        app.insert_resource(engine::spawn_engine());
        app.insert_resource(boxes::LatestSnapshot::default());
        app.insert_resource(boxes::PredictedSnapshot::default());
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

        let from = libp2p::PeerId::random();
        let remote_action = engine::Action::default();
        let remote_hash = engine::hash_action(&remote_action);
        for tick in 1..=40u64 {
            event_tx
                .send(p2p::Event::IncomingNetcode {
                    from,
                    msg: p2p::NetcodeMsg::Commit {
                        tick,
                        player_id: [2; 32],
                        hash: remote_hash,
                    },
                })
                .ok();
            event_tx
                .send(p2p::Event::IncomingNetcode {
                    from,
                    msg: p2p::NetcodeMsg::Reveal {
                        tick,
                        player_id: [2; 32],
                        action: remote_action,
                    },
                })
                .ok();
            app.update();
        }

        let clock = app.world().resource::<boxes::SimState>().clock;
        assert!(clock >= 40);
        let latest = app.world().resource::<boxes::SimState>().latest_hash;
        assert!(
            latest.is_some(),
            "a tick is applied once both peers have revealed for it"
        );
        let snap = app.world().resource::<boxes::LatestSnapshot>().0.is_some();
        assert!(snap);
    }
}
