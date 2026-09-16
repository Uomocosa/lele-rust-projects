# MAINNET PROBLEMS 5 — the instant-join run (5s gates, TDD loops)

Follow-up to `MAINNET_PROBLEMS_4.md`. This session started from a user
complaint — *"the test is fully green but it should not be"* — and turned
into a full instant-join project: clicking an already-created room must show
all peers + room state within 5s (loading screen in between: not spawned,
clicks dropped, leave-only). Method: `definition-tdd` fast-red/slow-gate
loops. Slow-gate count on the hardened `rooms_rejoin` suite: **0/5 green**
at the time of writing (each red below names one defect; gates 1–6 ran).

Related: `MAINNET_PROBLEMS.md`, `_2`, `_3`, `_4`. E2E harness:
`tests/e2e_local/rooms_rejoin.rs`. Crate: `clicker/`.

## 0. User decisions recorded in this session

**Q1 — join data path.** User premise (paraphrased): freenet discovery
should maintain a constantly-updated `rooms:peers` tree; on join, do a quick
PEX exchange over libp2p for the peer list (plus whatever others just
discovered) and the app state, so the player is up and ready; afterwards both
discoveries keep running in the background — freenet for rooms+peers,
libp2p for updates on the current room.

Author's argued refinements (accepted implicitly by proceeding — push back
if any of these misread you):
1. This premise *is* the documented architecture (`README.md`: freenet
   roster contract per room + directory contract, libp2p gossip mirror +
   PEX + SyncReq/Ack). Nothing new needs discovering — the defect is that
   the join path waits for a *fresh blocking contract Get* instead of using
   the already-polled state plus one active PEX round at click.
2. PEX-at-join needs ≥1 connected peer. Cold start (zero connections) must
   still dial directory hints first (already happens), then PEX. So the
   click path is: hints (instant, local) → dial → PEX confirm (~1 RTT) →
   targeted `SyncReq`s → ready. ~1s total, no contract `Get` on the
   critical path.
3. PEX gives you what *one* peer knows (partial view). "All peers" still
   requires a union (directory + PEX + gossip), i.e. the Option-A hint
   design from the session — with PEX-at-join as the freshness mechanism
   instead of dial-outcome pruning. Ghosts (stale entries) resolve via
   immediate dials, not via the 300s staleness window.
4. App *state* does not ride PEX today (`PexResp` carries hints + rooms,
   not scores). Scores keep flowing over `SyncReq`/`SyncAck` + snapshots;
   the PEX round's job is to trigger *targeted* syncs, not to carry them.

**Q2 — hygiene.** User: wrong-update-code hygiene is good; after green,
focus on simplification — the code is too complex to build on. Agreed:
the join path now spans `JoinPending` / `JoinGate` / `ExpectedRx` /
`poll_expected` / `clear_pending` across Bevy plus `room_requests` /
`pending_switch` / `send_expected` in the discovery task. Simplification
waits for 5/5.

## 1. P0 — no loading gate (the original complaint)

`clear_pending` (`src/lobby/bevy_systems/clear_pending.rs`) exited loading
on roster *non-empty* (one entry, likely self) or after a 1s timeout
(`JOIN_TIMEOUT_SECS`). Spawning (`spawn_on_join`) and click intake
(`detect_click`) never consulted `JoinPending`. The spinner was decoration.
Meanwhile the e2e only gated join on `lobby={room}` in the log (written at
click time — zero readiness signal), a 300s converge window and 60s
`agree_within` lag. A joiner converging over minutes passed everything.

Fix (this session): `JoinPending` gates spawn + clicks; `JoinGate`
(`src/lobby/join_gate.rs`: `expected: Option<BTreeSet<String>>` from
discovery, `synced: Vec<SyncedPeer>` from merged `SyncAck`s) clears only on
full-roster-connected + full-scores-merged, with a `join ready` log line;
timeout expiry *stays on the spinner* (never half-joins); leave aborts via
the existing `leave_button` path. `JOIN_TIMEOUT_SECS` deleted.

## 2. P1 — menu clickable before the node joins mainnet (gate 1 red)

`create-ready-1` failed: at click time the embedded node hadn't joined the
ring (`directory connect failed … peer has not joined the network yet`, 5s
retry loop), so discovery was stuck pre-room and no click-path budget can
cover it. Fix: menu buttons inert until the first directory feed
(`DirectoryLive` resource, `src/lobby/directory_live.rs`, set in
`poll_directory` with a `menu live: directory connected` line); the e2e
waits for it before creating. Creator keeps a 60s budget (cold `Put`
physics); joins keep 5s.

## 3. P2 — resolve-loop quantization (found while fixing P1)

`resolve_room` slept 5s (`DIRECTORY_TICK_SECS`) per iteration, so a click
waited up to 5s before being *noticed*; `drive_roster` quantizes switches
to its 1s tick. Fix: `tokio::select!` on `room_requests` vs the tick sleep
in `resolve_room` (`src/discovery/run.rs`), shared `resolve_requested`
helper. The 1s `drive_roster` tick stays (inside budget).

## 4. P3 — pre-switch links invisible to the new room (gate 2 red)

Instance 2's libp2p link to instance 1 predated its room switch.
`dial_known` skips connected peers (no new `PeerConnected`), so Bevy's
roster never learned the peer for the new room, no `SyncReq` went out —
permanent pre-ready deadlock. Fix: on expected-set arrival,
`poll_expected` (`src/lobby/bevy_systems/poll_expected.rs`) pushes
synthetic `PeerConnected` events, letting the existing
`poll_roster`/`send_sync_req` pipeline handle them.

## 5. P4 — `PeerConnected` consumer starvation (gate 4/5 reds)

`poll_roster` (plugin, registered first) and `send_sync_req` (clicker) both
*consume* `PeerConnected`. Whoever runs first eats it; the other starves —
so either the roster stayed empty (no spawn, no heartbeat members) or no
`SyncReq` went out (no acks), depending on order. Dead end analyzed:
no consume/restore combination is order-free while `poll_roster` must count
exactly (sub-connection link counting breaks on duplicates *and* on
misses). Fix, two halves: (a) `send_sync_req`
(`src/clicker/bevy_systems/send_sync_req.rs`) *restores* the event and is
idempotent per live peer (`Local` set, cleared on disconnect) — safe to
circulate, exact in both orders; (b) `send_sync_heartbeat`
(`src/clicker/bevy_systems/send_sync_heartbeat.rs`) fires instantly on
roster *growth* instead of only every 5s, so sync no longer depends on
winning the event race at all. Plugin crate untouched (its link counting
stays exact). Side effect caught by TDD: the restore flooded the
connection-less turmoil sims with unbounded `SyncReq`s — fixed by the same
idempotency guard.

## 6. P5 — unbounded contract `Get` tail (gates 3/4/6 reds)

Same-shaped `Get` measured 336–622ms twice, then hung past two 5s kills.
Node-side trace of a hung case: subscription ACK in ~50ms, state delivery
never follows (routing/replication stall on a churned ring — NAT failures
everywhere in the logs). `recv_after_get`
(`src/discovery/recv_after_get.rs`) looped `recv_response()` with **no
timeout**, and `switch_room` awaited it, stalling the whole discovery loop
(no gossip/redial/announce meanwhile). Fixes: 10s `CONNECT_TIMEOUT_SECS`
(`src/discovery/constants.rs`) on the connect `Get`
(`Error::ResponseTimeout` already existed); `switch_room` returns success
and `drive_roster` retries across ticks via `pending_switch`
(`src/discovery/run.rs`) instead of consuming the request on one failure.
Note: the timeout bounds the hang but cannot fit a stalled `Get` into 5s —
see §9.

## 7. P6 — cross-run contamination (found while fixing P5)

Gate-3's "fresh" room `Get` returned 3 pre-existing slots: consecutive runs
created identically-named rooms under the same namespace with default
params → identical contract keys → ghost state from dead runs. Fix
(test-only): per-run unique `CLICKER_CONTRACT_PARAMS` (timestamp-micros +
pid as even-length hex) set in-test before spawning, so each run gets fresh
keys while sharing the directory for discovery.

## 8. P7 — rejoin `Get` stall (gate 6 red, current)

Fresh joins now pass 5s reliably (create + join-2 + join-3 green, baseline
`p1=16 p2=16 p3=15`); the rejoin's `Get` stalled the same way as §6 (sub
ACK fast, state never). Same key succeeded 30s earlier — per-node routing
luck dominates, ~1/3 of Gets observed hanging. This is the evidence behind
the Q1 fork: the authoritative `Get` cannot meet 5s under churn.

## 9. Open: making 5s structural (the Q1 work, not started)

Per §0: at switch, send `expected` = fresh hint-union (directory
publishers + room-tagged PEX + topic-tagged gossip — tagging gossip by
room in `drain_lobby_events` is still TODO), dial immediately, PEX-confirm
at join, targeted syncs; contract `Get` union-upgrades in background;
prune dial-failed peers (route `DialFailed` into discovery first —
currently only warned in `forward_events`, `src/main.rs:186-193`).
Bevy gate unchanged. Until then the 5s asserts stand as written; expect
tail reds on churned rings.

## 10. Lint-architecture fights (all resolved this session)

- **E028 vs Bevy**: `Resource` has no blanket impl for `Vec<T>`, but E028
  bans collection newtypes. Resolution: `JoinGate` multi-field struct
  (`expected` + `synced`), following the `PendingClicks` precedent —
  collections live in named multi-field structs, never bare newtypes.
- **E001 vs E016**: single-caller SystemParams can't live in the caller
  file (E001: one public item) nor bare in their own file (E016). Pattern
  used: struct + thin delegate in its file (`spawn_ctx.rs`), real logic in
  a private `<type>_<method>.rs` file (`spawn_ctx_gate_open.rs`); bundles
  that couldn't justify a file were eliminated (`LeaveCtx` → overlay
  cleanup moved to `despawn_leave`, which already runs `OnEnter(Menu)`).
- **`too_many_arguments`**: `JoinCtx`/`JoinRooms`/`SpawnCtx` bundles
  (precedent: `DialHub`, `ResolveCtx`); `main.rs` channel setup extracted
  to `channels()` after hitting 101/100 lines.
- **E020**: `crate::` paths banned outside `use` items — including in
  `#[cfg(test)]` modules (several test-only violations fixed by importing
  the domain module).

## 11. Incidental findings (open, not blocking the gate)

- **Entity-despawn storm at click**: ~48 `Entity despawned` + B0004
  hierarchy errors in one second at every room click (menu teardown vs
  queued spinner-attach commands). One-shot per click, cosmetic, but the
  known `despawn_on_leave` vs queued-commands race (`_4.md` §5) is the same
  family — candidate for the next fast test after 5/5.
- **`update_total_board` rebuild churn**: `total board rebuild` logs
  suggest frequent despawn+respawn cycles — possible contributor to the
  above; not investigated.
- **Turmoil sims flake under load**: `star_turmoil` failed twice
  pre-change (loaded machine), both sims once post-change (fixed by the
  `send_sync_req` idempotency guard). Keep an eye on sim timing margins.
- **Room-name reuse**: consecutive runs printed identical
  `room-1789564239` names despite epoch-based naming — mechanism
  unexplained (§7 mitigates consequences via unique params, not the cause).
- **Frozen-slot probe limits**: fresh joins assert own-slot constancy
  (≤1, allowing the join click itself); rejoins skip it (merge
  legitimately moves the slot) — documented in-test.
- **Baseline asymmetry**: gate-6 baseline `p3=15` vs `p1=p2=16` (one
  dropped drive click on instance 3) — within minimums, but drive
  reliability is worth watching.

## 12. Scoreboard

- Fast suite: **261 nextest green** (was 230) + clippy/fmt/lint/taxonomy/
  bevy-lint green, incl. ~25 new gate/ordering/idempotency tests.
- Slow gate (`freenet:run-rooms-rejoin`, hardened: 5s ready ×4 joins +
  60s create + frozen-slot probes + leave-during-loading probe):
  gate 1 RED (P1) → gate 2 RED (P3) → gate 3 RED (P5) → gate 4 RED (P4) →
  gate 5 RED (P4, event race) → gate 6 RED (P7, rejoin `Get` stall).
  **Count: 0/5.** Next: §9 per Q1, then 5/5.

## 13. Session 2026-09-16 — libp2p join handshake + quiet-period readiness (commit `733fd77`)

Method: `definition-tdd` fast-red/slow-gate, three vertical slices.
Contract wasm **unchanged** (still needed: no contract edits for any of this).

**Slice 1 — `WantJoin`/`Welcome` req/res handshake (Option 2).**
New `CursorMsg::{WantJoin{room}, Welcome{room, peers, own_score, joining}}`
(`src/clicker/cursor_msg.rs:32-41`) + three Bevy systems:
`send_want_join.rs` (idempotent per-peer ask on connect, event restored
like `send_sync_req`), `answer_join.rs` (replies with roster names + own
authoritative score only, wrong-room dropped),
`absorb_welcome.rs` (unions sender+peers into roster/`JoinGate.expected`,
converts the score into a synthetic `SyncAck` so the existing
`absorb_sync` merge path does the work — zero merge duplication).
`main.rs:242-252` routes both to discovery (consumed in a later slice).
TDD incident: `Welcome` re-added a mesh-harness roster entry under a
second (`blake3`) key, resurrecting a despawned slot and REDDING
`rejoin::leaver_rejoins_at_old_score` — fixed with value-dedupe on insert
(same ghost family as `_4` iter-2).

**Slice 2a — spectate.** `spawn_on_join.rs` no longer gates on
`JoinPending`: remotes render while loading, own clicks stay parked
(`detect_click` unchanged). Deleted the dead gate
(`spawn_ctx_gate_open.rs`, `SpawnCtx::pending`); E016 then forced the
single-caller `SpawnCtx` bundle inline into `spawn_on_join`
(`LeaveCtx` precedent).

**Slice 2b — quiet-period readiness (the e2e defect).** First slow gate
RED: `Welcome`-union advertises peers the joiner can never dial/sync, and
the strict full-roster gate stalled forever (no `join ready` on
instance-2 while game traffic flowed). New `JoinClock{clicked_at,
last_new_peer}` (`src/lobby/join_clock.rs`) + `QUIET_SECS=1` /
`JOIN_CAP_SECS=30` (`src/lobby/constants.rs:3-4`): `clear_pending.rs`
clears on 1s-silence-since-last-*new*-peer or 30s cap; growth-only bumps
in `absorb_welcome`/`poll_expected` (duplicates never reset quiet).
Ghost pollution is harmless by construction: spawn partial, upgrade in
background.

## 14. Scoreboard after `733fd77`

- Fast suite: **273 nextest green** (was 261) + task-clippy/fmt/lint/
  bevy-lint/taxonomy green. New: `tests/mesh/join.rs` (handshake,
  roster+gate union, spectate), `clears_ghost_once_quiet`,
  `clears_on_alone_cap`, handshake unit tests.
- Slow gate (`freenet:run-rooms-rejoin`): **PASS (583s)** — first green
  with the new protocol. Prior 0/5 tail reds absorbed by the quiet gate.
- Known deviations: scores merge eagerly during spectate (not strict
  positions-only); `Welcome.joining` wired but always empty; no reply
  jitter; roster `Get` still on the click path (§9 slices 3+5 remain).
- Pre-existing, untouched: `cargo clippy --features dev` fails in
  `setup.rs`, `total_board.rs`, `testing/*` (canonical no-dev task
  clippy is clean).
