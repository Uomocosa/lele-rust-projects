use std::time::{SystemTime, UNIX_EPOCH};

use avian2d::prelude::{LinearVelocity, Position};
use bevy::prelude::*;

use freenet_libp2p_bevy_plugin::{net_id, p2p, roster};

use crate::boxes;

pub fn send_snapshot(
    commands: ResMut<p2p::P2pCommands<boxes::Payload>>,
    mut tick: ResMut<p2p::SnapshotTick>,
    identity: Res<net_id::LocalIdentity>,
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
        from_id: **player,
        tick: **tick,
        sent_at_ms,
        payload: boxes::Payload {
            x: position.x,
            y: position.y,
            vx: velocity.x,
            vy: velocity.y,
        },
    };

    for (id, entry) in roster.iter() {
        if *id == **identity {
            continue;
        }
        commands
            .send(p2p::Command::SendSnapshot {
                peer_id: entry.peer_id.clone(),
                snapshot: snapshot.clone(),
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
    use freenet_libp2p_bevy_plugin::{net_id, p2p, roster};

    #[test]
    fn test_usage() {
        let (cmd_tx, mut cmd_rx) =
            tokio::sync::mpsc::unbounded_channel::<p2p::Command<boxes::Payload>>();
        let mut app = App::new();
        app.insert_resource(net_id::LocalIdentity(net_id::NetworkId(1)));
        let mut entries = roster::RosterState::new();
        entries.insert(
            net_id::NetworkId(2),
            roster::PeerEntry {
                peer_id: "peer-2".to_string(),
                addrs: vec![],
                updated_at: 1,
            },
        );
        entries.insert(
            net_id::NetworkId(1),
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
            boxes::Player(net_id::NetworkId(1)),
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
                    && snapshot.from_id == net_id::NetworkId(1)
                    && snapshot.payload.x == 3.0
                    && snapshot.payload.y == 4.0
                    && snapshot.tick == 1
        ));
    }

    #[test]
    fn no_snapshot_without_local_box() {
        let (cmd_tx, mut cmd_rx) =
            tokio::sync::mpsc::unbounded_channel::<p2p::Command<boxes::Payload>>();
        let mut app = App::new();
        app.insert_resource(net_id::LocalIdentity(net_id::NetworkId(1)));
        app.insert_resource(roster::Roster::default());
        app.insert_resource(p2p::SnapshotTick::default());
        app.insert_resource(p2p::P2pCommands(cmd_tx));
        app.add_systems(Update, send_snapshot);

        app.update();

        assert!(cmd_rx.try_recv().is_err());
    }
}
