# 12. Leave doesn't stick, dead cursors, fail-deadline kills — MITIGATED

Follow-up to: `MAINNET_PROBLEMS_9.md`, `MAINNET_PROBLEMS_10.md` slices 2–4, `MAINNET_PROBLEMS_12.md` (R1/R2), `MAINNET_PROBLEMS_13.md` F1.
Related: `mainnet_problems/10-join-handshake-welcome.md`.

## Symptoms

1. **Leave does not stick — instance 4 auto-rejoins.** Joins, clicks
   leave, reaches menu, returns to the room on its own; `player=4`
   resolved on every survivor to the end.
2. **Fail deadline despawns a healthy labeled cursor.** After 31s at
   `players=1,2,3`, `players` drops to `1,2` permanently with `WARN
   clicker: join failed: no identity before the fail deadline
   peer=<player-3 transport>`; every later `(NetworkId(3), 15)` parks
   (31×) — transport healthy, merge has nowhere to put it.
3. **Own cursor persists in the Menu** (`sync lobby=` still reports
   local `PlayerNo`).

## Evidence (logs)

- Run `.local-run/rooms-rejoin-20260919-113027/`, task stdout
  `/tmp/opencode/e2e5.log` (`/tmp` may be wiped; instance logs
  primary):
  - `instance-4.log:97` `11:34:18.016 sync lobby= p1=0 p2=0 p3=0
    players=4 global=0` (menu) → `instance-4.log:600`
    `11:34:31.111 sync lobby=room-1789817443 p1=15 p2=15 p3=15
    players=1,2,3,4 global=45` (in-room again, no UI action).
  - Tails: `instance-1-rejoin.log:739`, `instance-2.log:3329`
    `players=1,2,3,4 global=45` to the end.
- Run `.local-run/rooms-rejoin-20260919-182713/` (158s) + `sync-2.log`
  (403 lines): `label_slot` resolved `player=3` at 18:28:09; at
  18:28:40 `players` → `1,2` with the `join failed` WARN above; then
  `sync: entry parked pending id=NetworkId(3) count=15` (31×).
- `_13` runs 1/3: tails `players=1,2,3,4 global=46` (run 1) /
  `global=47` (run 3, ghost carries the join click) while 16+15+15 =
  46; leaver itself clean (`instance-4.log` tail empty lobby).

## Root causes

1. `leave_room` cleared local state + `State::Menu`
   (`src/lobby/bevy_systems/leave_button.rs:27-29`,
   `src/lobby/leave_room.rs`) but not `RoomRx`/`SelectedRoom`;
   `apply_room` (`src/lobby/bevy_systems/apply_room.rs:13-27`) re-read
   the stale watch value next frame and re-joined.
2. `gate.pending` armed at handshake; `resolve_player.rs` labeled the
   skeleton on gossip `Move` but never disarmed pending →
   `spawn_pending_join.rs:28-40` inserted `PendingReveal{player:None}`
   onto the already-labeled slot → `reveal_on_join.rs:43-49` despawned
   the healthy cursor at `fail_at`, pushed peer to `gate.absent`;
   `spawn_on_join` refuses absent peers, so recovery is impossible.
3. `spawn_cursor` runs at `Startup`; only `despawn_leave` removes.

## Fixes (with src:line)

- **R1 — resolve disarms:** `resolve_player` labeling `Owner` X drops
  `gate.pending` entries mapping to X.
- **R2 — never re-arm a labeled slot:** `spawn_pending_join` skips
  entities already carrying `PlayerNo`; `reveal_on_join` never kills
  labeled slots.
- Leave-sticks (`_10` slice 2): `leave_room` clears `ActiveLobby` +
  `SelectedRoom` + counters + tombstones, records room in new
  `LeftRoom` (`src/lobby/left_room.rs`); `apply_room` ignores feed
  rooms equal to `LeftRoom`; deliberate menu joins clear it. `RoomRx`
  untouched (holds the discovery-owned `Receiver`).
- Menu cursor (`_10` slice 4): `despawn_leave` (`OnEnter(Menu)`)
  removes all `PlayerNo` entities incl. own
  (`own_cursor_removed_for_menu`).
- One-cursor-per-id (`_10` slice 3): audit — every spawn site already
  guards on `Owner`; pinned with `no_duplicate_owner_entities`.

## Verification

- `_12` killers absent 5 runs: `join failed` 0×, absent-marks 0×,
  `p3=0`-forever 0× (`_13` campaign). Frozen-gate relaxed to
  `first <= 1 && all-equal` (create click counts).
- Decision-log pipeline verified (`XtermSpec.decision_log` →
  `CLICKER_DECISION_LOG` → `drain_decisions` → `sync-N.log`).

## Open / next slice

- **F1 residual:** survivor side still shows the zero-count (or
  join-click) player-4 ghost — leaver side fixed (empty-lobby tails),
  survivor side needs a leave broadcast (no `PeerDisconnected` fires
  on deliberate leave). Demands the leaver-removal e2e assert first
  (`_9` §6.2: after `ui_leave_room`, survivors' resolved set must drop
  the leaver).
