use std::collections::HashSet;

use bevy::prelude::*;

use crate::boxes;

/// Positions every box purely from the engine's snapshots so the display app never runs its own
/// physics. The local box tracks the rollback session's *predicted* (immediate) state while remote
/// boxes come from the authoritative (committed) state. Creates a visual box for each engine body
/// (marking the local player) and despawns any visual box whose engine body is gone.
pub fn render_snapshots(
    mut commands: Commands,
    snapshot: Res<boxes::LatestSnapshot>,
    predicted: Res<boxes::PredictedSnapshot>,
    config: Res<boxes::Config>,
    existing: Query<(Entity, &boxes::Player, Option<&boxes::LocalPlayer>)>,
) {
    let own = **config;
    let Some(snapshot_value) = &**snapshot else {
        return;
    };
    let keep: HashSet<boxes::PlayerId> = snapshot_value.bodies.keys().copied().collect();
    let tick = snapshot_value.tick;
    let local_predicted: Option<(f32, f32)> = (**predicted)
        .as_ref()
        .and_then(|p| p.bodies.get(&own).copied());
    let by_id: std::collections::HashMap<boxes::PlayerId, Entity> = existing
        .iter()
        .map(|(entity, player, _)| (**player, entity))
        .collect();
    let is_local_by_state: std::collections::HashMap<boxes::PlayerId, bool> = existing
        .iter()
        .map(|(_, player, local)| (**player, local.is_some()))
        .collect();

    for (id, (x, y)) in &snapshot_value.bodies {
        let (render_x, render_y) = if *id == own {
            local_predicted.unwrap_or((*x, *y))
        } else {
            (*x, *y)
        };
        let position = Vec2::new(render_x, render_y);
        if let Some(&entity) = by_id.get(id) {
            commands.entity(entity).insert(Transform::from_translation(
                position.extend(Transform::default().translation.z),
            ));
            if *id == own && !is_local_by_state.get(id).copied().unwrap_or(false) {
                commands.entity(entity).insert(boxes::LocalPlayer);
            } else if *id != own && is_local_by_state.get(id).copied().unwrap_or(false) {
                commands.entity(entity).remove::<boxes::LocalPlayer>();
            }
        } else {
            boxes::spawn_box(&mut commands, boxes::Player(*id), position, *id == own);
        }

        if *id == own {
            tracing::debug!(
                target: "p2p",
                player_id = %hex::encode(id),
                x = position.x,
                y = position.y,
                tick,
                "sending engine snapshot"
            );
        }
    }

    for (entity, player, _) in existing.iter() {
        if !keep.contains(&**player) {
            commands.entity(entity).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use bevy::prelude::*;

    use super::render_snapshots;
    use crate::boxes;
    use crate::engine::Snapshot;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.insert_resource(boxes::Config::new([1; 32]));
        let mut bodies = BTreeMap::new();
        bodies.insert([2; 32], (10.0, 20.0));
        bodies.insert([1; 32], (30.0, 40.0));
        app.insert_resource(boxes::LatestSnapshot(Some(Snapshot { tick: 1, bodies })));
        app.insert_resource(boxes::PredictedSnapshot::default());
        app.add_systems(Update, render_snapshots);
        app.update();

        let mut query = app
            .world_mut()
            .query::<(&boxes::Player, Option<&boxes::LocalPlayer>)>();
        let mut local = 0;
        let mut total = 0;
        for (player, local_marker) in query.iter(app.world()) {
            total += 1;
            if local_marker.is_some() {
                assert_eq!(**player, [1; 32]);
                local += 1;
            }
        }
        assert_eq!(total, 2);
        assert_eq!(local, 1);
    }

    #[test]
    fn box_positions_come_from_engine_snapshot() {
        let mut app = App::new();
        app.insert_resource(boxes::Config::new([9; 32]));
        let mut bodies = BTreeMap::new();
        bodies.insert([9; 32], (5.0, 6.0));
        app.insert_resource(boxes::LatestSnapshot(Some(Snapshot { tick: 1, bodies })));
        app.insert_resource(boxes::PredictedSnapshot::default());
        app.add_systems(Update, render_snapshots);
        app.update();

        let (_, transform) = app
            .world_mut()
            .query::<(&boxes::Player, &Transform)>()
            .single(app.world())
            .unwrap();
        assert_eq!(
            (transform.translation.x, transform.translation.y),
            (5.0, 6.0)
        );
    }

    #[test]
    fn no_snapshot_renders_nothing() {
        let mut app = App::new();
        app.insert_resource(boxes::Config::new([9; 32]));
        app.insert_resource(boxes::LatestSnapshot::default());
        app.insert_resource(boxes::PredictedSnapshot::default());
        app.add_systems(Update, render_snapshots);
        app.update();

        let count = app
            .world_mut()
            .query::<&boxes::Player>()
            .iter(app.world())
            .count();
        assert_eq!(count, 0);
    }
}
