# MAINNET PROBLEMS 10 — layer rule + ≤500ms converge bar (slices 1–5)

Follow-up to `MAINNET_PROBLEMS_9.md`. Standing correction for all earlier
`MAINNET_PROBLEMS*.md` files, plus the work plan for the next slices.
Status notes below describe what actually landed (see `_11.md` for the
slow-gate result).

> **OUTDATED wording (kept for history):** passages in `_1`–`_9` saying
> "Freenet dials / Freenet addresses / discover each other via Freenet /
> hairpin blocks Freenet" mean **libp2p dials seeded by Freenet**.
> Freenet = rooms + occupants seed only (`src/discovery/peer_entry.rs`,
> `src/discovery/directory_entry.rs`); all dials/connections/updates =
> libp2p (`src/discovery/run.rs` drive loop, `CursorMsg` in `src/clicker/`).
> Ring-topology notes refer to node bootstrap, not gameplay.
> Room converge bar: **≤500ms same-host**.

## Layer rule

- FREENET (seed): room-name → params; room → `{PlayerNo → libp2p PeerId}`.
  `addrs` fields are one-time dial hints, not connection state.
- LIBP2P (transport): dial, PEX, gossip, `SyncReq`/`SyncAck`,
  `WantJoin`/`Welcome`, heartbeats. All gameplay delays live here.
- FREENET RING (`freenet-gateway` skill): node bootstrap plumbing only.
  That skill is correct for Freenet in general; in clicker sessions load it
  for harness-bootstrap questions only, never for app-mesh diagnosis.

## Converge bar

Post-drive room agreement must hold within **≤500ms** on same-host runs.
Ring-join may take seconds; room-converge may not. The e2e keeps a 60s
outer sampling window (`LAG_AGREE_TIMEOUT_SECS`) but reports the measured
time-to-agree against the 500ms budget (`baseline-converge-ms` check).

## Slice order

1. Docs (this file + `README.md` layer box). No `src/`. Done.
2. Leave-sticks: `leave_room` clears `ActiveLobby` + `SelectedRoom` +
   counters + tombstones and records the room in new `LeftRoom`
   (`src/lobby/left_room.rs`); `apply_room` ignores feed rooms equal to
   `LeftRoom`, deliberate menu joins clear it. `RoomRx` itself is
   untouched — it holds only the discovery-owned `Receiver`, so dropping
   it would blind future auto-joins. RED: leave → N `Update`s with
   `apply_room` running → `State::Menu` sticks. Done.
3. One-cursor-per-id: audit found every spawn site already guards on
   `Owner` (`spawn_on_join` known-vec, `spawn_pending_join` lookup;
   resolve/merge paths never spawn). Skeleton cursors carry the peer id
   as `Owner(NetworkId::from_peer(..))`. Pinned with regression test
   `no_duplicate_owner_entities`, no product change. Done.
4. Menu cursor: `despawn_leave` (`OnEnter(Menu)`) removes all `PlayerNo`
   entities incl. own. Pinned with `own_cursor_removed_for_menu`.
   Menu = OS cursor, in-room = app cursor. Done.
5. ≤500ms converge: `agree_within` poll tightened 500ms→50ms;
   `phase_converge_drive` reports measured time-to-agree as
   `baseline-converge-ms` against `CONVERGE_BUDGET_MS = 500` (soft, run
   continues). Hunt `_9 §2` lag in the libp2p sync path only
   (`send_sync_*`, `absorb_snapshot`, `apply_delta`). Measurement done;
   hunt open — see `_11.md`.

Standing rules: negative assertions per phase, budget-with-measurement
comments, scenario-fidelity check before GREEN.
