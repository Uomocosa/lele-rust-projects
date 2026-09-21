use bevy::prelude::*;

use freenet_libp2p_bevy_plugin::{net_id, p2p};

use crate::clicker;
use crate::lobby;

pub fn decide_join_commit(
    mut gate: ResMut<lobby::JoinGate>,
    clock: Res<lobby::JoinClock>,
    lobby: Res<clicker::ActiveLobby>,
    own: Res<net_id::NetworkId>,
    commands: ResMut<p2p::Commands<clicker::CursorMsg>>,
) {
    let clock = clock.into_inner();
    let own = *own.into_inner();
    if clock.clicked_at.is_none() || gate.has_pending(own) || gate.committed {
        return;
    }
    let room = (**lobby).clone();
    if room.is_empty() {
        return;
    }
    if !ready(&gate, clock) {
        return;
    }
    gate.committed = true;
    let now = std::time::Instant::now();
    let reveal_at = now
        .checked_add(std::time::Duration::from_millis(lobby::JOIN_REVEAL_MS))
        .unwrap_or(now);
    let fail_at = now
        .checked_add(std::time::Duration::from_secs(
            lobby::PENDING_JOIN_FAIL_SECS,
        ))
        .unwrap_or(now);
    gate.arm_pending(lobby::PendingJoin {
        peer: format!("self-{}", *own),
        joiner: own,
        reveal_at,
        fail_at,
    });
    let msg = clicker::CursorMsg::JoinCommit {
        room: room.clone(),
        joiner: own,
        starts_in_ms: lobby::JOIN_REVEAL_MS,
    };
    let data = bincode::serialize(&msg).unwrap_or_default();
    commands.into_inner().push(p2p::Command::Publish {
        topic: clicker::pos_topic(lobby.into_inner()),
        data,
    });
    clicker::DecisionLog::record(&format!("commit: sent joiner={} room={room}", *own));
    tracing::info!(target: "clicker", room = %room, joiner = *own, "join commit sent");
}

// needed helper: all expected peers acked, or the commit cap elapsed
fn ready(gate: &lobby::JoinGate, clock: &lobby::JoinClock) -> bool {
    let synced = |peer: &String| gate.synced.iter().any(|entry| entry.as_str() == peer);
    let all_synced = gate
        .expected
        .as_ref()
        .is_some_and(|wanted| wanted.iter().all(synced));
    let capped = clock.clicked_at.is_some_and(|at| {
        std::time::Instant::now()
            .saturating_duration_since(at)
            .as_secs()
            >= lobby::JOIN_COMMIT_CAP_SECS
    });
    all_synced || capped
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use bevy::prelude::*;

    use super::decide_join_commit;
    use crate::clicker;
    use crate::lobby;
    use freenet_libp2p_bevy_plugin::{net_id, p2p};

    fn test_app(expected: BTreeSet<String>, synced: Vec<&str>) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(clicker::ActiveLobby("alpha".to_string()));
        app.insert_resource(net_id::NetworkId(1));
        app.insert_resource(lobby::JoinPending(Some("alpha".to_string())));
        app.insert_resource(lobby::JoinClock {
            clicked_at: Some(std::time::Instant::now()),
            last_new_peer: None,
        });
        app.insert_resource(lobby::JoinGate {
            expected: Some(expected),
            synced: synced
                .into_iter()
                .map(|peer| lobby::SyncedPeer(peer.to_string()))
                .collect(),
            committed: false,
            pending: Vec::new(),
            absent: Vec::new(),
        });
        app.insert_resource(p2p::Commands::<clicker::CursorMsg>::default());
        app
    }

    #[test]
    fn test_usage() {
        let mut app = test_app(
            BTreeSet::from(["peer-2".to_string(), "peer-3".to_string()]),
            vec!["peer-2", "peer-3"],
        );
        app.add_systems(Update, decide_join_commit);
        app.update();
        assert!(
            app.world()
                .resource::<lobby::JoinGate>()
                .has_pending(net_id::NetworkId(1)),
            "all peers acked, own reveal armed"
        );
        let commands = app.world().resource::<p2p::Commands<clicker::CursorMsg>>();
        assert!(
            commands.iter().any(|command| matches!(
                command,
                p2p::Command::Publish { data, .. }
                    if matches!(bincode::deserialize::<clicker::CursorMsg>(data), Ok(clicker::CursorMsg::JoinCommit { .. }))
            )),
            "commit is broadcast"
        );
    }

    #[test]
    fn waits_while_a_peer_is_silent() {
        let mut app = test_app(
            BTreeSet::from(["peer-2".to_string(), "peer-3".to_string()]),
            vec!["peer-2"],
        );
        app.add_systems(Update, decide_join_commit);
        app.update();
        assert_eq!(app.world().resource::<lobby::JoinGate>().pending.len(), 0);
        assert!(
            app.world()
                .resource::<p2p::Commands<clicker::CursorMsg>>()
                .is_empty(),
            "no commit while a peer is still silent"
        );
    }

    #[test]
    fn commits_once_not_twice() {
        let mut app = test_app(BTreeSet::from(["peer-2".to_string()]), vec!["peer-2"]);
        app.add_systems(Update, decide_join_commit);
        app.update();
        app.update();
        assert_eq!(
            app.world()
                .resource::<p2p::Commands<clicker::CursorMsg>>()
                .len(),
            1,
            "commit is broadcast once"
        );
    }

    #[test]
    fn commits_after_join_pending_cleared() {
        let mut app = test_app(BTreeSet::new(), Vec::new());
        app.insert_resource(lobby::JoinPending::default());
        app.add_systems(Update, decide_join_commit);
        app.update();
        assert!(
            app.world().resource::<lobby::JoinGate>().committed,
            "the commit still fires after the loading gate cleared"
        );
    }
}
