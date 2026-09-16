use std::collections::BTreeSet;

use bevy::prelude::*;
use freenet_libp2p_bevy_plugin::roster;

use crate::lobby;

pub fn clear_pending(
    mut commands: Commands,
    pending: ResMut<lobby::JoinPending>,
    gate: Res<lobby::JoinGate>,
    members: Res<roster::Roster>,
    overlays: Query<Entity, With<lobby::bevy_systems::LoadingRoot>>,
    spinners: Query<Entity, With<lobby::bevy_systems::JoinSpinner>>,
) {
    let Some(room) = (**pending).clone() else {
        return;
    };
    let gate = gate.into_inner();
    let members = members.into_inner();
    let Some(wanted) = gate.expected.clone() else {
        return;
    };
    if !is_ready(members, &gate.synced, &room, &wanted) {
        return;
    }
    tracing::info!(target: "clicker", room = %room, peers = wanted.len(), "join ready: full roster synced");
    let pending = pending.into_inner();
    **pending = None;
    for entity in &overlays {
        commands.entity(entity).despawn();
    }
    for entity in &spinners {
        commands.entity(entity).despawn();
    }
}

// needed helper: true once every expected peer is in the roster and has synced scores
fn is_ready(
    members: &roster::Roster,
    synced: &[lobby::SyncedPeer],
    room: &str,
    wanted: &BTreeSet<String>,
) -> bool {
    let present: BTreeSet<&str> = members
        .get(room)
        .map(|entries| entries.values().map(String::as_str).collect())
        .unwrap_or_default();
    wanted.iter().all(|peer| {
        present.contains(peer.as_str())
            && synced.iter().any(|entry| entry.as_str() == peer.as_str())
    })
}

#[cfg(test)]
mod tests {
    use super::clear_pending;
    use crate::lobby;
    use bevy::prelude::*;
    use freenet_libp2p_bevy_plugin::roster;
    use std::collections::BTreeSet;

    fn test_app(room: &str, gate: lobby::JoinGate) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(lobby::JoinPending(Some(room.to_string())));
        app.insert_resource(gate);
        app.insert_resource(roster::Roster::default());
        app.add_systems(Update, clear_pending);
        app
    }

    // needed helper: builds a gate holding only an expected set
    fn gate_with(wanted: Option<BTreeSet<String>>) -> lobby::JoinGate {
        lobby::JoinGate {
            expected: wanted,
            synced: Vec::new(),
        }
    }

    // needed helper: stages one roster entry in the test world
    fn add_roster(app: &mut App, room: &str, peer: &str) {
        let key = *blake3::hash(peer.as_bytes()).as_bytes();
        app.world_mut().resource_mut::<roster::Roster>().add_entry(
            room.to_string(),
            key,
            peer.to_string(),
        );
    }

    // needed helper: stages one synced peer in the test world
    fn mark_synced(app: &mut App, peer: &str) {
        app.world_mut()
            .resource_mut::<lobby::JoinGate>()
            .synced
            .push(lobby::SyncedPeer(peer.to_string()));
    }

    #[test]
    fn stays_while_expected_unknown() {
        let mut app = test_app("room-a", gate_with(None));
        add_roster(&mut app, "room-a", "peer-2");
        app.update();
        assert!(
            app.world().resource::<lobby::JoinPending>().is_some(),
            "loading persists without the expected set"
        );
    }

    #[test]
    fn stays_on_partial_roster() {
        let wanted: BTreeSet<String> = ["peer-2".to_string(), "peer-3".to_string()]
            .into_iter()
            .collect();
        let mut app = test_app("room-a", gate_with(Some(wanted)));
        add_roster(&mut app, "room-a", "peer-2");
        mark_synced(&mut app, "peer-2");
        app.update();
        assert!(
            app.world().resource::<lobby::JoinPending>().is_some(),
            "loading persists until every peer is visible"
        );
    }

    #[test]
    fn stays_on_partial_sync() {
        let wanted: BTreeSet<String> = ["peer-2".to_string(), "peer-3".to_string()]
            .into_iter()
            .collect();
        let mut app = test_app("room-a", gate_with(Some(wanted)));
        add_roster(&mut app, "room-a", "peer-2");
        add_roster(&mut app, "room-a", "peer-3");
        mark_synced(&mut app, "peer-2");
        app.update();
        assert!(
            app.world().resource::<lobby::JoinPending>().is_some(),
            "loading persists until every peer synced scores"
        );
    }

    #[test]
    fn test_usage() {
        let wanted: BTreeSet<String> = ["peer-2".to_string(), "peer-3".to_string()]
            .into_iter()
            .collect();
        let mut app = test_app("room-a", gate_with(Some(wanted)));
        add_roster(&mut app, "room-a", "peer-2");
        add_roster(&mut app, "room-a", "peer-3");
        mark_synced(&mut app, "peer-2");
        mark_synced(&mut app, "peer-3");
        let overlay = app.world_mut().spawn(lobby::bevy_systems::LoadingRoot).id();
        let spinner = app.world_mut().spawn(lobby::bevy_systems::JoinSpinner).id();
        app.update();
        assert!(
            app.world().resource::<lobby::JoinPending>().is_none(),
            "ready joiner leaves loading"
        );
        assert!(
            app.world().get_entity(overlay).is_err(),
            "overlay removed once join completes"
        );
        assert!(
            app.world().get_entity(spinner).is_err(),
            "spinner removed once join completes"
        );
    }

    #[test]
    fn creator_with_empty_expected_clears() {
        let mut app = test_app("room-a", gate_with(Some(BTreeSet::new())));
        app.update();
        assert!(
            app.world().resource::<lobby::JoinPending>().is_none(),
            "empty room is ready at once"
        );
    }
}
