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
    let room = (**lobby).clone();
    let gate = gate.into_inner();
    let now = std::time::Instant::now();
    for (entity, owner, marker) in &targets {
        if now < marker.reveal_at {
            continue;
        }
        let held = gate
            .pending
            .iter()
            .find(|entry| target_owner(entry, own) == **owner)
            .map(|entry| (entry.peer.clone(), entry.joiner));
        if let Some(player) = marker.player {
            clicker::label_slot(
                &mut commands,
                &mut materials,
                entity,
                held.as_ref().map_or("", |(peer, _)| peer.as_str()),
                player,
            );
            commands.entity(entity).remove::<clicker::PendingReveal>();
            if let Some((_, joiner)) = &held {
                gate.pending.retain(|entry| entry.joiner != *joiner);
            }
            tracing::info!(target: "clicker", room = %room, player, "join reveal");
        } else if now >= marker.fail_at {
            commands.entity(entity).despawn();
            if let Some((peer, joiner)) = &held {
                gate.absent.push(peer.clone());
                gate.pending.retain(|entry| entry.joiner != *joiner);
                tracing::warn!(target: "clicker", room = %room, peer = %peer, "join failed: no identity before the fail deadline");
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

    use super::reveal_on_join;
    use crate::clicker;
    use crate::lobby;
    use freenet_libp2p_bevy_plugin::net_id;

    fn marker(
        reveal_at: std::time::Instant,
        fail_at: std::time::Instant,
        player: Option<u64>,
    ) -> clicker::PendingReveal {
        clicker::PendingReveal {
            reveal_at,
            fail_at,
            player,
        }
    }

    fn test_app(
        reveal_at: std::time::Instant,
        fail_at: std::time::Instant,
        player: Option<u64>,
    ) -> (App, Entity) {
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
            fail_at,
        });
        app.insert_resource(gate);
        let entity = app
            .world_mut()
            .spawn((
                clicker::CursorIcon,
                clicker::Owner(net_id::NetworkId::from_peer("peer-2")),
                clicker::CursorColor(Color::srgb(0.5, 0.5, 0.5)),
                clicker::ClickCounter::default(),
                marker(reveal_at, fail_at, player),
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
        let (mut app, entity) = test_app(past, past, Some(2));
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
        let now = std::time::Instant::now();
        let future = now
            .checked_add(std::time::Duration::from_secs(30))
            .unwrap_or(now);
        let (mut app, entity) = test_app(future, future, Some(2));
        app.update();
        assert!(app.world().get::<clicker::PlayerNo>(entity).is_none());
        assert!(app.world().get::<clicker::PendingReveal>(entity).is_some());
        assert_eq!(app.world().resource::<lobby::JoinGate>().pending.len(), 1);
    }

    #[test]
    fn late_identity_reveals_without_early_expiry() {
        let now = std::time::Instant::now();
        let reveal_at = now
            .checked_sub(std::time::Duration::from_secs(5))
            .unwrap_or(now);
        let fail_at = now
            .checked_add(std::time::Duration::from_secs(25))
            .unwrap_or(now);
        let (mut app, entity) = test_app(reveal_at, fail_at, None);
        app.update();
        assert!(
            app.world().get_entity(entity).is_ok(),
            "no identity yet, but still within the fail window"
        );
        assert!(app.world().get::<clicker::PlayerNo>(entity).is_none());
    }

    #[test]
    fn expired_identity_despawns_and_marks_absent() {
        let past = std::time::Instant::now()
            .checked_sub(std::time::Duration::from_secs(1))
            .unwrap_or_else(std::time::Instant::now);
        let (mut app, entity) = test_app(past, past, None);
        app.update();
        assert!(
            app.world().get_entity(entity).is_err(),
            "unresolved placeholder despawned past the fail deadline"
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
    fn unknown_identity_stays_gray_until_fail() {
        let now = std::time::Instant::now();
        let reveal_at = now
            .checked_sub(std::time::Duration::from_secs(1))
            .unwrap_or(now);
        let fail_at = now
            .checked_add(std::time::Duration::from_secs(30))
            .unwrap_or(now);
        let (mut app, entity) = test_app(reveal_at, fail_at, None);
        app.update();
        assert!(app.world().get_entity(entity).is_ok());
        assert!(app.world().get::<clicker::PlayerNo>(entity).is_none());
    }
}
