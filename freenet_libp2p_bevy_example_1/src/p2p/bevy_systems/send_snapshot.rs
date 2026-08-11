use std::time::{SystemTime, UNIX_EPOCH};

use avian2d::prelude::{LinearVelocity, Position};
use bevy::prelude::*;

use crate::boxes;
use crate::p2p;
use crate::roster;

pub fn send_snapshot(
    commands: ResMut<p2p::P2pCommands>,
    mut tick: ResMut<p2p::SnapshotTick>,
    config: Res<boxes::Config>,
    roster: Res<roster::Roster>,
    local_box: Query<(&boxes::Player, &Position, &LinearVelocity), With<boxes::LocalPlayer>>,
) {
    let Ok((player, position, velocity)) = local_box.single() else {
        return;
    };

    **tick += 1;
    let sent_at_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    let snapshot = p2p::Snapshot {
        player_id: ***player,
        x: position.x,
        y: position.y,
        vx: velocity.x,
        vy: velocity.y,
        tick: **tick,
        sent_at_ms,
    };

    for (player_id, entry) in roster.iter() {
        if *player_id == **config {
            continue;
        }
        commands
            .send(p2p::Command::SendSnapshot {
                peer_id: entry.peer_id.clone(),
                snapshot,
            })
            .ok();
    }
}

#[cfg(test)]
mod tests {
    use avian2d::prelude::{LinearVelocity, Position};
    use bevy::prelude::*;

    use super::send_snapshot;
    use crate::boxes;
    use crate::p2p;
    use crate::roster;

    #[test]
    fn test_usage() {
        let (cmd_tx, mut cmd_rx) = tokio::sync::mpsc::unbounded_channel::<p2p::Command>();
        let mut app = App::new();
        app.insert_resource(boxes::Config::new(boxes::PlayerId(1)));
        let mut entries = roster::RosterState::new();
        entries.insert(
            boxes::PlayerId(2),
            roster::PeerEntry {
                peer_id: "peer-2".to_string(),
                addrs: vec![],
                updated_at: 1,
            },
        );
        entries.insert(
            boxes::PlayerId(1),
            roster::PeerEntry {
                peer_id: "self".to_string(),
                addrs: vec![],
                updated_at: 1,
            },
        );
        app.insert_resource(roster::Roster(entries));
        app.insert_resource(p2p::SnapshotTick::default());
        app.insert_resource(p2p::P2pCommands(cmd_tx));
        app.world_mut().spawn((
            boxes::Player(boxes::PlayerId(1)),
            boxes::LocalPlayer,
            Position::from_xy(3.0, 4.0),
            LinearVelocity::from(Vec2::new(1.0, 2.0)),
        ));
        app.add_systems(Update, send_snapshot);

        app.update();

        assert!(matches!(
            cmd_rx.try_recv(),
            Ok(p2p::Command::SendSnapshot { peer_id, snapshot })
                if peer_id == "peer-2"
                    && snapshot.player_id == 1
                    && snapshot.x == 3.0
                    && snapshot.y == 4.0
                    && snapshot.tick == 1
        ));
    }
}
