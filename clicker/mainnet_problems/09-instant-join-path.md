# 09. Instant-join path (P0–P5) — MITIGATED

Follow-up to: `MAINNET_PROBLEMS_5.md` §§1–9, `§§13–14` (handshake + quiet period).
Related: `mainnet_problems/08-roster-get-stall.md`, `mainnet_problems/10-join-handshake-welcome.md`.

Joining an already-created room must show all peers + room state
within 5s (loading screen in between: not spawned, clicks dropped,
leave-only). Slow-gate count at time of writing: 0/5 green → PASS
(583s) after `733fd77`.

## Symptoms & fixes

- **P0 — no loading gate (the original complaint).**
  `clear_pending` (`src/lobby/bevy_systems/clear_pending.rs`) exited
  loading on roster non-empty or 1s timeout (`JOIN_TIMEOUT_SECS`);
  spawn/clicks never consulted `JoinPending`.
  Fix: `JoinPending` gates spawn + clicks; `JoinGate`
  (`src/lobby/join_gate.rs`: `expected` + `synced`) clears only on
  full-roster-connected + full-scores-merged (`join ready` line);
  timeout expiry stays on the spinner; `JOIN_TIMEOUT_SECS` deleted.
- **P1 — menu clickable before ring join (gate 1 red).**
  `create-ready-1` failed: node hadn't joined the ring
  (`directory connect failed … peer has not joined the network yet`).
  Fix: menu buttons inert until first directory feed
  (`DirectoryLive`, `src/lobby/directory_live.rs`, `menu live:
  directory connected` line); e2e waits for it. Creator keeps 60s
  budget; joins keep 5s.
- **P2 — resolve-loop quantization.**
  `resolve_room` slept 5s (`DIRECTORY_TICK_SECS`) per iteration.
  Fix: `tokio::select!` on `room_requests` vs tick sleep in
  `resolve_room` (`src/discovery/run.rs`), shared
  `resolve_requested` helper. 1s `drive_roster` tick stays.
- **P3 — pre-switch links invisible to the new room (gate 2 red).**
  Libp2p link predated room switch; `dial_known` skips connected
  peers (no new `PeerConnected`), roster never learns the peer.
  Fix: on expected-set arrival, `poll_expected`
  (`src/lobby/bevy_systems/poll_expected.rs`) pushes synthetic
  `PeerConnected` events.
- **P4 — `PeerConnected` consumer starvation (gates 4/5 red).**
  `poll_roster` (plugin) and `send_sync_req` (clicker) both consume
  `PeerConnected`; whoever runs first starves the other.
  Fix: `send_sync_req` (`src/clicker/bevy_systems/send_sync_req.rs`)
  *restores* the event + idempotent per-live-peer (`Local` set,
  cleared on disconnect); `send_sync_heartbeat`
  (`src/clicker/bevy_systems/send_sync_heartbeat.rs`) fires on roster
  *growth*, not only every 5s. Plugin crate untouched.
- **P5 — unbounded contract `Get` tail** → see
  `08-roster-get-stall.md` (10s timeout + `pending_switch` retry).
- **Cross-run contamination:** identical room names + default params
  → identical keys → ghost state. Fix (test-only): unique
  `CLICKER_CONTRACT_PARAMS` per run.

## Evidence (logs)

- Gates 1–6 RED sequence (`_5` §12): gate 1 RED (P1) → gate 2 RED
  (P3) → gate 3 RED (P5) → gate 4 RED (P4) → gate 5 RED (P4 race) →
  gate 6 RED (P7 rejoin `Get` stall). Run dirs under
  `.local-run/rooms-rejoin-*` (deleted since).
- Fast suite: 261 → 273 nextest green, ~25 new gate/ordering/
  idempotency tests; `tests/mesh/join.rs` (handshake, roster+gate
  union, spectate).

## Verification

- Slow gate (`freenet:run-rooms-rejoin`): PASS (583s) after `733fd77`.

## Open / next slice

- Roster `Get` off the click path (`08-roster-get-stall.md` §Open).
- Simplification pass (user-accepted, deferred past 5/5): join path
  spans `JoinPending`/`JoinGate`/`ExpectedRx`/`poll_expected`/
  `clear_pending` + `room_requests`/`pending_switch`/`send_expected`.
