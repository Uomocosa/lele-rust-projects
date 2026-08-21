use std::collections::HashSet;

use bevy::prelude::*;

use crate::boxes;
use crate::roster;

pub fn spawn_roster_boxes(
    mut commands: Commands,
    roster: Res<roster::Roster>,
    existing: Query<(&boxes::Player, &Transform)>,
) {
    if !roster.is_changed() {
        return;
    }

    let spawned: HashSet<boxes::PlayerId> = existing.iter().map(|(player, _)| **player).collect();
    let mut occupied_xs: Vec<f32> = existing
        .iter()
        .map(|(_, transform)| transform.translation.x)
        .collect();

    for id in roster.keys() {
        if spawned.contains(id) {
            continue;
        }
        let x = boxes::pick_spawn_x(&occupied_xs);
        occupied_xs.push(x);
        tracing::debug!(
            target: "roster",
            player = format!("{:08x}", **id as u32),
            x = x,
            "spawning box for player"
        );
        boxes::spawn_box(
            &mut commands,
            boxes::Player(*id),
            Vec2::new(x, boxes::SPAWN_Y),
            false,
        );
    }
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use avian2d::prelude::RigidBody;
    use bevy::prelude::*;

    use super::spawn_roster_boxes;
    use crate::boxes;
    use crate::roster;

    fn now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }

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

    fn entry(peer_id: &str, updated_at: u64) -> roster::PeerEntry {
        roster::PeerEntry {
            peer_id: peer_id.to_string(),
            addrs: vec![],
            updated_at,
        }
    }

    fn build_app(
        own_id: boxes::PlayerId,
        entries: roster::RosterState,
    ) -> (App, tokio::sync::mpsc::UnboundedSender<roster::Event>) {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        tx.send(roster::Event::Roster { entries }).unwrap();
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::transform::TransformPlugin);
        app.insert_resource(ButtonInput::<KeyCode>::default());
        app.add_plugins(boxes::Plugin(boxes::Config::new(own_id)));
        app.add_plugins(roster::Plugin(roster::Config::new(rx)));
        (app, tx)
    }

    fn count_boxes(app: &mut App) -> (usize, usize, usize) {
        let mut query = app
            .world_mut()
            .query::<(&boxes::Player, Option<&boxes::LocalPlayer>, &RigidBody)>();
        let mut total = 0;
        let mut local = 0;
        let mut kinematic = 0;
        for (_player, local_marker, body) in query.iter(app.world()) {
            total += 1;
            if local_marker.is_some() {
                local += 1;
            }
            if *body == RigidBody::Kinematic {
                kinematic += 1;
            }
        }
        (total, local, kinematic)
    }

    #[test]
    fn single_clean_session_has_one_box() {
        let own_id = boxes::PlayerId(7);
        let mut entries = roster::RosterState::default();
        entries.insert(own_id, entry("self", now()));

        let (mut app, _tx) = build_app(own_id, entries);
        app.update();

        assert_eq!(count_boxes(&mut app), (1, 1, 0));
    }

    #[test]
    fn stale_previous_session_entry_does_not_duplicate() {
        let own_id = boxes::PlayerId(7);
        let mut entries = roster::RosterState::default();
        entries.insert(own_id, entry("self", now()));
        entries.insert(
            boxes::PlayerId(99),
            entry("stale-session", now() - roster::ROSTER_ENTRY_TTL_SECS - 10),
        );

        let (mut app, _tx) = build_app(own_id, entries);
        app.update();

        assert_eq!(count_boxes(&mut app), (1, 1, 0));
    }

    #[test]
    fn two_players_spawn_one_box_each() {
        let own_id = boxes::PlayerId(7);
        let mut entries = roster::RosterState::default();
        entries.insert(own_id, entry("self", now()));
        entries.insert(boxes::PlayerId(8), entry("live-peer", now()));

        let (mut app, _tx) = build_app(own_id, entries);
        app.update();

        assert_eq!(count_boxes(&mut app), (2, 1, 1));
    }

    #[test]
    fn boxes_spawn_spread_out() {
        let own_id = boxes::PlayerId(7);
        let mut entries = roster::RosterState::default();
        entries.insert(own_id, entry("self", now()));
        entries.insert(boxes::PlayerId(8), entry("live-peer", now()));

        let (mut app, _tx) = build_app(own_id, entries);
        app.update();

        let mut query = app.world_mut().query::<(&boxes::Player, &Transform)>();
        let mut xs: Vec<f32> = query
            .iter(app.world())
            .map(|(_, t)| t.translation.x)
            .collect();
        xs.sort_by(f32::total_cmp);
        assert_eq!(xs.len(), 2);
        assert!((xs[0] - xs[1]).abs() > f32::EPSILON);
    }
}
