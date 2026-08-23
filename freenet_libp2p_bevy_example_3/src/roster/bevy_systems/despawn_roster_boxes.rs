use bevy::prelude::*;

use crate::boxes;
use crate::roster;

/// Removes boxes for players who have left the roster.
///
/// `spawn_roster_boxes` only ever adds, so without this a peer that went stale kept its box
/// forever and the screen stopped matching the roster resource. That mattered beyond
/// cosmetics: screenshots were being used as evidence about roster state while the two
/// layers could disagree.
///
/// The local player is never despawned — its box is spawned by `boxes::bevy_systems::setup`
/// before the roster has loaded, so an empty or not-yet-populated roster must not remove it.
pub fn despawn_roster_boxes(
    mut commands: Commands,
    roster: Res<roster::Roster>,
    existing: Query<(Entity, &boxes::Player), Without<boxes::LocalPlayer>>,
) {
    if !roster.is_changed() || roster.is_empty() {
        return;
    }

    for (entity, player) in existing.iter() {
        if roster.contains_key(&**player) {
            continue;
        }
        let key = *player;
        tracing::debug!(
            target: "roster",
            player = format!("{:08x}", u32::from_le_bytes([key[0], key[1], key[2], key[3]])),
            "despawning box for departed player"
        );
        commands.entity(entity).despawn();
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::despawn_roster_boxes;
    use crate::boxes;
    use crate::roster;

    fn entry(peer_id: &str) -> roster::PeerEntry {
        roster::PeerEntry {
            peer_id: peer_id.to_string(),
            addrs: vec![],
            seq: 1,
            signature: Vec::new(),
        }
    }

    fn count_players(app: &mut App) -> usize {
        let mut query = app.world_mut().query::<&boxes::Player>();
        query.iter(app.world()).count()
    }

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.world_mut().spawn(boxes::Player([1; 32]));
        app.world_mut().spawn(boxes::Player([2; 32]));

        let mut entries = roster::RosterState::default();
        entries.insert([1; 32], entry("peer-1"));
        app.insert_resource(roster::Roster(entries));
        app.add_systems(Update, despawn_roster_boxes);
        app.update();

        assert_eq!(count_players(&mut app), 1);
    }

    #[test]
    fn local_player_is_never_despawned() {
        let mut app = App::new();
        app.world_mut()
            .spawn((boxes::Player([7; 32]), boxes::LocalPlayer));

        let mut entries = roster::RosterState::default();
        entries.insert([9; 32], entry("someone-else"));
        app.insert_resource(roster::Roster(entries));
        app.add_systems(Update, despawn_roster_boxes);
        app.update();

        assert_eq!(count_players(&mut app), 1);
    }

    #[test]
    fn empty_roster_despawns_nothing() {
        let mut app = App::new();
        app.world_mut().spawn(boxes::Player([1; 32]));

        app.insert_resource(roster::Roster(roster::RosterState::default()));
        app.add_systems(Update, despawn_roster_boxes);
        app.update();

        assert_eq!(count_players(&mut app), 1);
    }
}
