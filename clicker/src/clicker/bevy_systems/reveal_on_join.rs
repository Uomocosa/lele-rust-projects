use bevy::prelude::*;

use freenet_libp2p_bevy_plugin::net_id;

use crate::clicker;
use crate::lobby;

pub fn reveal_on_join(
    mut commands: Commands,
    gate: ResMut<lobby::JoinGate>,
    own: Res<net_id::NetworkId>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    lobby: Res<clicker::ActiveLobby>,
    targets: Query<(Entity, &clicker::Owner, &clicker::PendingReveal)>,
) {
    let own = *own.into_inner();
    let lobby = lobby.into_inner();
    let gate = gate.into_inner();
    let now = std::time::Instant::now();
    for entry in gate.take_due_pending(now) {
        let target = target_owner(&entry, own);
        let mut found = false;
        for (entity, owner, marker) in &targets {
            if **owner != target {
                continue;
            }
            found = true;
            if let Some(player) = marker.player {
                clicker::label_slot(&mut commands, &mut materials, entity, &entry.peer, player);
                commands.entity(entity).remove::<clicker::PendingReveal>();
                tracing::info!(target: "clicker", room = %**lobby, player, "join reveal");
            } else {
                commands.entity(entity).despawn();
                gate.absent.push(entry.peer.clone());
                tracing::warn!(target: "clicker", peer = %entry.peer, "join failed: no identity before reveal");
            }
            break;
        }
        if !found {
            gate.absent.push(entry.peer.clone());
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

    use super::reveal_on_join;
    use crate::clicker;
    use crate::lobby;
    use freenet_libp2p_bevy_plugin::net_id;

    fn test_app(reveal_at: std::time::Instant) -> (App, Entity) {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Assets<ColorMaterial>>();
        app.insert_resource(clicker::ActiveLobby("alpha".to_string()));
        app.insert_resource(net_id::NetworkId(1));
        let mut gate = lobby::JoinGate::default();
        gate.arm_pending(lobby::PendingJoin {
            peer: "peer-2".to_string(),
            joiner: net_id::NetworkId(2),
            reveal_at,
        });
        app.insert_resource(gate);
        let entity = app
            .world_mut()
            .spawn((
                clicker::CursorIcon,
                clicker::Owner(net_id::NetworkId::from_peer("peer-2")),
                clicker::CursorColor(Color::srgb(0.5, 0.5, 0.5)),
                clicker::ClickCounter::default(),
                clicker::PendingReveal {
                    reveal_at,
                    player: Some(2),
                },
            ))
            .id();
        app.add_systems(Update, reveal_on_join);
        (app, entity)
    }

    #[test]
    fn test_usage() {
        let past = std::time::Instant::now()
            .checked_sub(std::time::Duration::from_secs(1))
            .unwrap_or_else(std::time::Instant::now);
        let (mut app, entity) = test_app(past);
        app.update();
        app.update();
        let player = app.world().get::<clicker::PlayerNo>(entity);
        assert_eq!(player.map(|no| **no), Some(2), "revealed with its number");
        assert!(
            app.world().get::<clicker::PendingReveal>(entity).is_none(),
            "pending marker cleared"
        );
        assert_eq!(app.world().resource::<lobby::JoinGate>().pending.len(), 0);
    }

    #[test]
    fn not_due_stays_hidden() {
        let future = std::time::Instant::now()
            .checked_add(std::time::Duration::from_secs(30))
            .unwrap_or_else(std::time::Instant::now);
        let (mut app, entity) = test_app(future);
        app.update();
        assert!(app.world().get::<clicker::PlayerNo>(entity).is_none());
        assert!(app.world().get::<clicker::PendingReveal>(entity).is_some());
        assert_eq!(app.world().resource::<lobby::JoinGate>().pending.len(), 1);
    }

    #[test]
    fn expired_identity_despawns_and_marks_absent() {
        let past = std::time::Instant::now()
            .checked_sub(std::time::Duration::from_secs(1))
            .unwrap_or_else(std::time::Instant::now);
        let (mut app, entity) = test_app(past);
        app.world_mut()
            .entity_mut(entity)
            .insert(clicker::PendingReveal {
                reveal_at: past,
                player: None,
            });
        app.update();
        assert!(
            app.world().get_entity(entity).is_err(),
            "unresolved placeholder despawned"
        );
        assert!(
            app.world()
                .resource::<lobby::JoinGate>()
                .absent
                .contains(&"peer-2".to_string()),
            "the failed peer is remembered absent"
        );
    }

    #[test]
    fn unknown_identity_stays_gray_until_due() {
        let future = std::time::Instant::now()
            .checked_add(std::time::Duration::from_secs(30))
            .unwrap_or_else(std::time::Instant::now);
        let (mut app, entity) = test_app(future);
        app.world_mut()
            .entity_mut(entity)
            .insert(clicker::PendingReveal {
                reveal_at: future,
                player: None,
            });
        app.update();
        assert!(app.world().get_entity(entity).is_ok());
        assert!(app.world().get::<clicker::PlayerNo>(entity).is_none());
    }
}
