# 17. Menu double-despawn warnings at room transitions — MITIGATED

Related: `mainnet_problems/14-turmoil-rig-fidelity.md` (log fidelity), `mainnet_problems/16-rejoin-reveal-log-flake.md` (clean logs needed for the reveal decision trail).

## Symptom

Every room/menu transition spams `bevy_ecs::error::handler`:

```
WARN bevy_ecs::error::handler: Encountered an error in command
`… entity_command::despawn::…`: Entity despawned: The entity with ID
441v0 is invalid; its index now has generation 1.
```

~N-1 warnings per menu rebuild, where N is the number of menu nodes.
Non-fatal, but it drowns the decision log the rejoin work needs.

## Evidence (logs)

- `clicker/.local-run/rooms-rejoin-20260921-092745/instance-2.log:184`
  (09:28:16.303) and `:190`, `:196` — a burst of invalid-despawn
  warnings; another burst at `:216` (09:28:18.394).
- `clicker/.local-run/rooms-rejoin-20260921-093159/instance-2.log`
  (09:32:31.796) — ~15 consecutive warnings, one per menu child.

## Root cause

`show_menu` tags the root node **and every child** with `MenuRoot`
(`src/lobby/bevy_systems/show_menu.rs:25,41,46,57,62,74`). Bevy 0.19
`Commands::entity(e).despawn()` is recursive, so:

- `despawn_menu` (`src/lobby/bevy_systems/despawn_menu.rs:5-12`) iterates
  all `MenuRoot` entities; the first (root) despawns the whole tree,
  then each remaining child command hits a dead entity → warning.
- the `show_menu` rebuild loop (`show_menu.rs:12-18`) has the same shape.

`join_feedback` also attaches a `JoinSpinner` as a **child of a
`MenuRoot` RoomButton** (`src/lobby/bevy_systems/join_feedback.rs:29-40`),
so menu teardown and `clear_pending` can race over the same subtree.

## Fix

Decision **3A** (landed): despawn only parentless menu roots and let
recursion remove children.

```rust
// src/lobby/bevy_systems/despawn_menu.rs:5
pub fn despawn_menu(
    mut commands: Commands,
    menus: Query<Entity, (With<lobby::bevy_systems::MenuRoot>, Without<ChildOf>)>,
) {
    for entity in &menus {
        commands.entity(entity).despawn();
    }
}
```

The same `Without<ChildOf>` filter is on the `show_menu` rebuild query
(`show_menu.rs:7`). Regression tests install a panicking Bevy error
handler (`app.set_error_handler(bevy::ecs::error::panic)`) and assert a
nested menu tree despawns once:
`despawn_menu::tests::{test_usage, nested_menu_nodes_despawn_once}`.

Result: the ~15-warning-per-transition spam is gone. Two residual
warnings remain in `instance-4.log` during the leave-during-loading
phase (`rooms-rejoin-20260921-131928`), so the row stays MITIGATED
until those are traced (likely `clear_pending` + `despawn_leave`
racing over the LoadingRoot/spinner).

## Verification

- Fast: `lele:clippy/fmt/nextest/lint/bevy-lint/taxonomy_check` green
  (386/386); the new tests panic before the fix and pass after.
- Slow: `freenet:run-rooms-rejoin` — per-transition spam gone; 2
  residual warnings in `instance-4.log` (leave-load path).

## Open / next slice

- Trace the 2 residual `instance-4.log` despawns (leave-load path).
- 3B (root-only `MenuRoot` tag) if more child coupling appears.
