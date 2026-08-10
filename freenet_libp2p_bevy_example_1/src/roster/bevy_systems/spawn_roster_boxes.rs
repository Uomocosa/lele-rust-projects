use std::collections::HashSet;

use bevy::prelude::*;

use crate::boxes;
use crate::roster;

pub fn spawn_roster_boxes(
    mut commands: Commands,
    roster: Res<roster::Roster>,
    existing: Query<&boxes::Player>,
) {
    if !roster.is_changed() {
        return;
    }

    let spawned: HashSet<boxes::PlayerId> = existing.iter().map(|player| **player).collect();

    for id in roster.keys() {
        if spawned.contains(id) {
            continue;
        }
        boxes::spawn_box(
            &mut commands,
            boxes::Player(*id),
            Vec2::new(0.0, boxes::GROUND_Y + boxes::BOX_SIZE),
            false,
        );
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::spawn_roster_boxes;
    use crate::boxes;
    use crate::roster;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        let mut entries = roster::RosterState::default();
        entries.insert(
            boxes::PlayerId(1),
            roster::PeerEntry {
                peer_id: "peer-1".to_string(),
                addrs: vec![],
                updated_at: 1,
            },
        );
        app.insert_resource(roster::Roster(entries));
        app.add_systems(Update, spawn_roster_boxes);
        app.update();

        let mut query = app.world_mut().query::<&boxes::Player>();
        assert_eq!(query.iter(app.world()).count(), 1);
    }
}
