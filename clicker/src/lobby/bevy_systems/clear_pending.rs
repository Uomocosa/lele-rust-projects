use bevy::prelude::*;
use freenet_libp2p_bevy_plugin::net_id;

use crate::clicker;
use crate::lobby;

type LoadingNodes<'w, 's> = Query<
    'w,
    's,
    Entity,
    Or<(
        With<lobby::bevy_systems::LoadingRoot>,
        With<lobby::bevy_systems::JoinSpinner>,
    )>,
>;

pub fn clear_pending(
    mut commands: Commands,
    pending: ResMut<lobby::JoinPending>,
    gate: Res<lobby::JoinGate>,
    clock: Res<lobby::JoinClock>,
    own: Res<net_id::NetworkId>,
    overlays: LoadingNodes<'_, '_>,
) {
    let Some(room) = (**pending).clone() else {
        return;
    };
    if gate.has_pending(*own.into_inner()) {
        return;
    }
    let gate = gate.into_inner();
    let clock = clock.into_inner();
    if let Some(wanted) = gate.expected.as_ref() {
        if !is_ready(wanted, clock) {
            return;
        }
        tracing::info!(target: "clicker", room = %room, peers = wanted.len(), "join ready: quiet roster settled");
    } else {
        if !is_capped(clock) {
            return;
        }
        tracing::warn!(target: "clicker", room = %room, "join abandoned: roster never resolved before the alone-cap");
    }
    clicker::DecisionLog::record(&format!(
        "join: clear room={room} committed={}",
        gate.committed
    ));
    finish_join(&mut commands, pending, &overlays);
}

// needed helper: ready once discovery goes quiet or the alone-cap expires
fn is_ready(wanted: &std::collections::BTreeSet<String>, clock: &lobby::JoinClock) -> bool {
    if wanted.is_empty() {
        return true;
    }
    let now = std::time::Instant::now();
    let quiet = clock.last_new_peer.is_some_and(|at| {
        now.checked_duration_since(at)
            .is_none_or(|d| d.as_secs() >= lobby::QUIET_SECS)
    });
    let capped = clock.clicked_at.is_some_and(|at| {
        now.checked_duration_since(at)
            .is_none_or(|d| d.as_secs() >= lobby::JOIN_CAP_SECS)
    });
    quiet || capped
}

// needed helper: true once the alone-cap elapsed without a resolved roster
fn is_capped(clock: &lobby::JoinClock) -> bool {
    clock.clicked_at.is_some_and(|at| {
        std::time::Instant::now()
            .checked_duration_since(at)
            .is_none_or(|d| d.as_secs() >= lobby::JOIN_CAP_SECS)
    })
}

// needed helper: drops the pending gate and its loading overlay/spinner
fn finish_join(
    commands: &mut Commands,
    pending: ResMut<lobby::JoinPending>,
    overlays: &LoadingNodes<'_, '_>,
) {
    let pending = pending.into_inner();
    **pending = None;
    for entity in overlays {
        commands.entity(entity).despawn();
    }
}

#[cfg(test)]
mod tests {
    use super::clear_pending;
    use crate::lobby;
    use bevy::prelude::*;
    use std::collections::BTreeSet;

    fn test_app(room: &str, gate: lobby::JoinGate, clock: lobby::JoinClock) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(lobby::JoinPending(Some(room.to_string())));
        app.insert_resource(gate);
        app.insert_resource(clock);
        app.insert_resource(freenet_libp2p_bevy_plugin::net_id::NetworkId(1));
        app.add_systems(Update, clear_pending);
        app
    }

    // needed helper: builds a gate holding only an expected set
    fn gate_with(wanted: Option<BTreeSet<String>>) -> lobby::JoinGate {
        lobby::JoinGate {
            expected: wanted,
            synced: Vec::new(),
            committed: false,
            pending: Vec::new(),
            absent: Vec::new(),
        }
    }

    // needed helper: clock quiet long ago, clicked just now
    fn quiet_clock() -> lobby::JoinClock {
        lobby::JoinClock {
            clicked_at: Some(std::time::Instant::now()),
            last_new_peer: Some(
                std::time::Instant::now()
                    .checked_sub(std::time::Duration::from_secs(lobby::QUIET_SECS + 1))
                    .unwrap_or_else(std::time::Instant::now),
            ),
        }
    }

    // needed helper: clock busy right now
    fn busy_clock() -> lobby::JoinClock {
        lobby::JoinClock {
            clicked_at: Some(std::time::Instant::now()),
            last_new_peer: Some(std::time::Instant::now()),
        }
    }

    // needed helper: clock past the alone-cap with no peers discovered
    fn capped_clock() -> lobby::JoinClock {
        lobby::JoinClock {
            clicked_at: Some(
                std::time::Instant::now()
                    .checked_sub(std::time::Duration::from_secs(lobby::JOIN_CAP_SECS + 1))
                    .unwrap_or_else(std::time::Instant::now),
            ),
            last_new_peer: None,
        }
    }

    #[test]
    fn stays_while_expected_unknown() {
        let mut app = test_app("room-a", gate_with(None), busy_clock());
        app.update();
        assert!(
            app.world().resource::<lobby::JoinPending>().is_some(),
            "loading persists without the expected set"
        );
    }

    #[test]
    fn abandons_when_expected_never_arrives() {
        let mut app = test_app("room-a", gate_with(None), capped_clock());
        let overlay = app.world_mut().spawn(lobby::bevy_systems::LoadingRoot).id();
        app.update();
        assert!(
            app.world().resource::<lobby::JoinPending>().is_none(),
            "unresolved roster releases the gate past the alone-cap"
        );
        assert!(
            app.world().get_entity(overlay).is_err(),
            "overlay removed when the join is abandoned"
        );
    }

    #[test]
    fn stays_while_discovering() {
        let wanted: BTreeSet<String> = ["peer-2".to_string(), "peer-3".to_string()]
            .into_iter()
            .collect();
        let mut app = test_app("room-a", gate_with(Some(wanted)), busy_clock());
        app.update();
        assert!(
            app.world().resource::<lobby::JoinPending>().is_some(),
            "loading persists while new peers still arrive"
        );
    }

    #[test]
    fn clears_once_quiet() {
        let wanted: BTreeSet<String> = ["peer-2".to_string(), "peer-3".to_string()]
            .into_iter()
            .collect();
        let mut app = test_app("room-a", gate_with(Some(wanted)), quiet_clock());
        let overlay = app.world_mut().spawn(lobby::bevy_systems::LoadingRoot).id();
        let spinner = app.world_mut().spawn(lobby::bevy_systems::JoinSpinner).id();
        app.update();
        assert!(
            app.world().resource::<lobby::JoinPending>().is_none(),
            "quiet joiner leaves loading without waiting for stragglers"
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
    fn clears_ghost_once_quiet() {
        let wanted: BTreeSet<String> = std::iter::once("peer-ghost".to_string()).collect();
        let mut app = test_app("room-a", gate_with(Some(wanted)), quiet_clock());
        app.update();
        assert!(
            app.world().resource::<lobby::JoinPending>().is_none(),
            "undialable welcome peers never stall the gate past quiet"
        );
    }

    #[test]
    fn clears_on_alone_cap() {
        let wanted: BTreeSet<String> = std::iter::once("peer-ghost".to_string()).collect();
        let clock = lobby::JoinClock {
            clicked_at: Some(
                std::time::Instant::now()
                    .checked_sub(std::time::Duration::from_secs(lobby::JOIN_CAP_SECS + 1))
                    .unwrap_or_else(std::time::Instant::now),
            ),
            last_new_peer: Some(std::time::Instant::now()),
        };
        let mut app = test_app("room-a", gate_with(Some(wanted)), clock);
        app.update();
        assert!(
            app.world().resource::<lobby::JoinPending>().is_none(),
            "joiner alone too long spawns partial"
        );
    }

    #[test]
    fn test_usage() {
        let wanted: BTreeSet<String> = ["peer-2".to_string(), "peer-3".to_string()]
            .into_iter()
            .collect();
        let mut app = test_app("room-a", gate_with(Some(wanted)), quiet_clock());
        app.update();
        assert!(
            app.world().resource::<lobby::JoinPending>().is_none(),
            "ready joiner leaves loading"
        );
    }

    #[test]
    fn creator_with_empty_expected_clears() {
        let mut app = test_app("room-a", gate_with(Some(BTreeSet::new())), busy_clock());
        app.update();
        assert!(
            app.world().resource::<lobby::JoinPending>().is_none(),
            "empty room is ready at once"
        );
    }
}
