use bevy::input::mouse::MouseButton;
use bevy::prelude::*;

use freenet_libp2p_bevy_plugin::{net_id, p2p, roster};

use crate::clicker;

pub fn detect_click(
    mut targets: Query<
        (
            &clicker::Owner,
            &mut clicker::ClickCounter,
            &GlobalTransform,
        ),
        With<clicker::ClickTarget>,
    >,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    identity: Res<net_id::LocalIdentity>,
    commands: ResMut<p2p::P2pCommands<clicker::ClickDelta>>,
    roster: Res<roster::Roster>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    let Some(window) = windows.single().ok() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let size = Vec2::new(window.width(), window.height());
    let ndc = (cursor / size) * 2.0 - Vec2::ONE;
    let world = Vec3::new(ndc.x * 100.0, ndc.y * 100.0, 0.0);

    for (owner, mut counter, transform) in &mut targets {
        let pos = transform.translation().truncate();
        if world.truncate().distance(pos) >= clicker::CLICK_RADIUS {
            continue;
        }
        if **owner == **identity {
            **counter += 1;
            tracing::debug!(target: "clicker", count = **counter, "local click");
            let delta = clicker::ClickDelta(1);
            for (id, entry) in roster.iter() {
                if *id == **identity {
                    continue;
                }
                commands
                    .send(p2p::Command::SendSnapshot {
                        peer_id: entry.peer_id.clone(),
                        snapshot: p2p::Snapshot {
                            from_id: **owner,
                            tick: 0,
                            sent_at_ms: 0,
                            payload: delta,
                        },
                    })
                    .ok();
            }
        } else {
            **counter -= 1;
        }
        return;
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::detect_click;
    use crate::clicker;
    use freenet_libp2p_bevy_plugin::{net_id, p2p, roster};

    #[test]
    fn test_usage() {
        let (cmd_tx, mut cmd_rx) =
            tokio::sync::mpsc::unbounded_channel::<p2p::Command<clicker::ClickDelta>>();
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
        app.insert_resource(roster::Roster(entries));
        app.insert_resource(p2p::P2pCommands(cmd_tx));
        app.insert_resource(ButtonInput::<bevy::input::mouse::MouseButton>::default());
        app.world_mut().spawn((
            clicker::Owner(net_id::NetworkId(1)),
            clicker::ClickTarget,
            clicker::ClickCounter::default(),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));
        app.add_systems(Update, detect_click);
        app.update();

        assert!(cmd_rx.try_recv().is_err());
    }
}
