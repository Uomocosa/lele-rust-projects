#![allow(clippy::needless_pass_by_value)]
use bevy::prelude::*;

use super::super::constants;
use super::super::join_clock::JoinClock;
use super::super::join_gate::JoinGate;
use super::super::join_pending::JoinPending;
use super::super::loading_root::LoadingRoot;

pub fn clear_pending(
    mut commands: Commands,
    mut pending: ResMut<JoinPending>,
    gate: Res<JoinGate>,
    clock: Res<JoinClock>,
    overlays: Query<Entity, With<LoadingRoot>>,
) {
    if pending.is_none() {
        return;
    }
    if !is_ready(&gate, &clock) {
        return;
    }
    **pending = None;
    for entity in &overlays {
        commands.entity(entity).despawn();
    }
}

// needed helper: ready once discovery goes quiet or the alone-cap expires
fn is_ready(gate: &JoinGate, clock: &JoinClock) -> bool {
    let now = std::time::Instant::now();
    let quiet = clock.last_new_peer.is_some_and(|at| {
        now.checked_duration_since(at)
            .is_none_or(|d| d.as_secs() >= constants::QUIET_SECS)
    });
    let capped = clock.clicked_at.is_some_and(|at| {
        now.checked_duration_since(at)
            .is_none_or(|d| d.as_secs() >= constants::JOIN_CAP_SECS)
    });
    match gate.expected.as_ref() {
        Some(expected) if expected.is_empty() => true,
        Some(_) => quiet || capped,
        None => capped,
    }
}
// no test_usage necessary
