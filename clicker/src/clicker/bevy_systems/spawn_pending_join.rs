use bevy::prelude::*;

use freenet_libp2p_bevy_plugin::net_id;

use crate::clicker;
use crate::lobby;

pub fn spawn_pending_join(
    mut commands: Commands,
    gate: Res<lobby::JoinGate>,
    own: Res<net_id::NetworkId>,
    owners: Query<(Entity, &clicker::Owner, Option<&clicker::PendingReveal>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let own = *own.into_inner();
    let pending = &gate.into_inner().pending;
    let gray = Color::srgb(0.5, 0.5, 0.5);
    for entry in pending {
        let target = target_owner(entry, own);
        let mut existing = None;
        for (entity, owner, marker) in &owners {
            if **owner == target {
                existing = Some((entity, marker.is_some()));
                break;
            }
        }
        match existing {
            Some((_, true)) => {}
            Some((entity, false)) => {
                commands.entity(entity).insert((
                    clicker::PendingReveal {
                        reveal_at: entry.reveal_at,
                        player: (entry.joiner == own).then_some(*own),
                    },
                    clicker::CursorColor(gray),
                    MeshMaterial2d(materials.add(gray)),
                ));
                tracing::info!(target: "clicker", peer = %entry.peer, "pending join armed");
            }
            None => {
                let spot = clicker::spawn_spot(target);
                let fill = commands
                    .spawn((
                        clicker::CursorIcon,
                        clicker::Owner(target),
                        clicker::CursorColor(gray),
                        clicker::ClickCounter::default(),
                        clicker::TargetPos(spot),
                        clicker::PendingReveal {
                            reveal_at: entry.reveal_at,
                            player: (entry.joiner == own).then_some(*own),
                        },
                        Mesh2d(meshes.add(clicker::cursor_mesh(1.0))),
                        MeshMaterial2d(materials.add(gray)),
                        Transform::from_translation(Vec3::new(spot.x, spot.y, 10.0)),
                    ))
                    .id();
                commands.entity(fill).with_children(|parent| {
                    parent.spawn((
                        clicker::CursorLabel,
                        Text2d::new(clicker::math_formatter(0)),
                        Transform::from_translation(Vec3::new(-17.0, 34.0, 0.5)),
                    ));
                });
                tracing::info!(target: "clicker", peer = %entry.peer, "pending join spawned");
            }
        }
    }
}

// needed helper: the owner key an entry maps to (own logical id, or the peer hash)
fn target_owner(entry: &lobby::PendingJoin, own: net_id::NetworkId) -> net_id::NetworkId {
    if entry.joiner == own {
        own
    } else {
        net_id::NetworkId::from_peer(&entry.peer)
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::spawn_pending_join;
    use crate::clicker;
    use crate::lobby;
    use freenet_libp2p_bevy_plugin::net_id;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<ColorMaterial>>();
        app.insert_resource(lobby::JoinGate::default());
        app.insert_resource(net_id::NetworkId(1));
        app
    }

    fn arm(app: &mut App, peer: &str, joiner: u64) {
        let reveal_at = std::time::Instant::now()
            .checked_add(std::time::Duration::from_secs(5))
            .unwrap_or_else(std::time::Instant::now);
        app.world_mut()
            .resource_mut::<lobby::JoinGate>()
            .arm_pending(lobby::PendingJoin {
                peer: peer.to_string(),
                joiner: net_id::NetworkId(joiner),
                reveal_at,
            });
    }

    #[test]
    fn test_usage() {
        let mut app = test_app();
        arm(&mut app, "peer-2", 2);
        app.add_systems(Update, spawn_pending_join);
        app.update();
        app.update();
        let count = app
            .world_mut()
            .query::<&clicker::PendingReveal>()
            .iter(app.world())
            .count();
        assert_eq!(count, 1, "pending joiner gains a gray marker");
    }

    #[test]
    fn existing_own_cursor_is_regrayed_not_duplicated() {
        let mut app = test_app();
        let existing = app
            .world_mut()
            .spawn((
                clicker::CursorIcon,
                clicker::Owner(net_id::NetworkId(1)),
                clicker::PlayerNo(1),
                clicker::ClickCounter(7),
                clicker::CursorColor(clicker::color_for(net_id::NetworkId(1))),
            ))
            .id();
        arm(&mut app, "self-1", 1);
        app.add_systems(Update, spawn_pending_join);
        app.update();
        let cursor = app.world().get::<clicker::CursorColor>(existing);
        assert_eq!(
            cursor.map(|color| **color),
            Some(Color::srgb(0.5, 0.5, 0.5))
        );
        let count = app
            .world_mut()
            .query::<&clicker::Owner>()
            .iter(app.world())
            .count();
        assert_eq!(count, 1, "one cursor, not a duplicate");
    }

    #[test]
    fn already_armed_is_idempotent() {
        let mut app = test_app();
        arm(&mut app, "peer-2", 2);
        app.add_systems(Update, spawn_pending_join);
        app.update();
        app.update();
        let count = app
            .world_mut()
            .query::<&clicker::PendingReveal>()
            .iter(app.world())
            .count();
        assert_eq!(count, 1);
    }
}
