# MAINNET PROBLEMS 6 — turmoil fleet, retained totals, SyncAck tombstones

Follow-up to `MAINNET_PROBLEMS_5.md` (§§13–14: `WantJoin`/`Welcome`
handshake + quiet-period readiness, commit `733fd77`, first
`rooms_rejoin` PASS). This session stayed in fast tests: making turmoil
mimic real networks, and pinning the "total drops when a player leaves"
invariant. **No product fix landed — the run stops at a RED test.**

## 0. What was built (all in `clicker/tests/mesh/`, uncommitted)

- **`rstest = "=0.27.0"`** pinned (`clicker/Cargo.toml`, dev-deps). Nextest
  now lists per-case tests (`case_1_leave_lan`, `seed_1_7`, …) — the
  `pytest.mark.parametrize` equivalent. Neither `tokio` (`#[tokio::test]`
  only runs async fns) nor `tracing` (`#[instrument]` + log capture)
  offers case generation; `test-case` was considered and rejected (no
  value-matrix, no fixtures — saves nothing over a hand macro).
- **`net_profile.rs`** — `NetworkProfile { seed, min/max_latency,
  fail_rate, tcp_capacity, slow_pairs, topology, chaos }`,
  `Topology::{Star, FullMesh}`, `lanes_for()`, `build_sim()` onto
  turmoil `Builder` + per-pair `set_link_latency`/`set_link_fail_rate`.
  `FLEET[4]` (`lan`, `star-relay`, `flaky-nat`, `leave`) + `LEAVE_RELAY`.
- **`turmoil_rig.rs`** — unified rig extracted from both turmoil files:
  one `bring_up`/`handshake`/`converge_to`/`settle`, envelope IO with
  gossip dedup-forward. Converted `star_turmoil` (seeds 7,8,9) and
  `asymmetric_turmoil` (seeds 42,43) to thin `rstest` wrappers — same
  scenarios, all green.
- **Connection lifecycle in `pump`** — stream death synthesizes
  `PeerConnected`/`PeerDisconnected` + roster prune (the harness
  equivalent of `poll_roster`'s disconnect arm). `partition` stays a
  stall (no events; flood on repair), `crash`/early-return means
  disconnect — matching libp2p `ConnectionClosed` semantics.

## 1. Rig incident (fixed in-rig, instructive)

First rig version broke `star` on all seeds: a host future *returning*
(converged early, test artifact) drops its sockets, and the synthesis
faithfully pruned a live peer — survivors could never converge. Peers
now `park_until_done` after recording until the client releases them;
only genuine leavers disconnect. Rule: *completion artifact vs real
leave* is structural in the rig (`turmoil_rig.rs: park_until_done`).

## 2. The RED: total not retained after leave

`total_retained.rs` (`#[case::leave_lan]`, `#[case::leave_relay]`):
converge to (1,5,17), peer-3's host returns, survivors settle:

```
assertion `left == right` failed: survivor peer-1 lost the leaver's total
  left: 6
  right: 23
```

Both profiles, ~8s wall. Root chain (all current source):
`despawn_on_leave.rs:30-36` parks the score in tombstones,
`sync_global.rs:9-15` sums **live counters only**, tombstones are read
exactly once (`resolve_player.rs:56`, rejoin restore). Fix direction:
global = Σ live + Σ un-restored tombstones (monotonic per room;
`leave_room.rs:12` already clears on leave, so room-scoping holds).

## 3. Why the naive fix is insufficient (SyncAck must carry tombstones)

Payload asymmetry: `publish_snapshot.rs:31-61` already unions live +
tombstones (max per id) for kad snapshots, but `absorb_sync.rs:68-82`
`snapshot_entries` sends **live labeled slots only**. Worked case:
B partitioned from C pre-leave learns `{1:1, 2:5}` (global 6) while A
learns `{1:1, 2:5, 3:17}` (global 23); after C leaves, naive retention
gives A=23 vs B=6 — still disagreeing, e2e `agree_within` still red
with a new signature. Only A's post-leave `SyncAck` carrying its
tombstone `(3,17)` heals B to 23. `merge_entries` needs no changes
(unknown ids already park as absolute `PendingClick`s); `keep()` is
already max-merge, so replays are no-ops. Planned sketch:

```rust
SyncReq { requester } => {
    let mut entries = snapshot_entries(&targets);  // live, as today
    // + retained tombstones, same max-per-id union as publish_snapshot
    commands.push(Send { peer_id: from, payload: SyncAck { target: requester, entries } });
}
```

## 4. Turmoil honesty boundary (verified in source)

`send_sync_heartbeat.rs:28` and `publish_due.rs:9` gate on Bevy
wall-clock `Time`, which advances by real microseconds per
`app.update()` while virtual time races — periodic re-sync effectively
never fires in-sim. After `repair`, B's roster is unchanged and no new
`PeerConnected` arrives, so nothing re-asks A in-sim (production's 15s
redial/heartbeat would). A full diverge→leave→heal agreement test would
fail even with the payload fix — rig infidelity, not product bug.
Scoped accordingly: (a) now — turmoil asserts per-survivor monotonic
retention + unit tests for tombstone-carrying `SyncAck`; cross-survivor
agreement stays with the slow gate. (b) later — link-down detection in
the rig (last-seen threshold → synthesized disconnect, i.e. TCP
keepalive timeout) so repair drives the production `PeerConnected` →
`SyncReq` burst; deserves its own green-baseline-first loop.

## 5. Scoreboard / next steps for tomorrow

- Fast: `total_retained` 0/2 RED (the stop point); everything else
  untouched since the 273-green gate — full suite not re-run this
  session (no `src/` changes).
- Next loop: GREEN the RED — tombstone-inclusive `sync_global` +
  tombstone-carrying `SyncAck` + merge/retention unit tests → fast-full
  gates → slow gate (`freenet:run-rooms-rejoin`) as the only oracle
  (retention needs pre-leave convergence; max-merge heals, but fixed
  e2e windows may sample mid-heal).
- Open threads: reply jitter + `Welcome.joining` (slice 3), roster `Get`
  off the click path (slice 5), rig link-down detection (§4b),
  `cargo clippy --features dev` pre-existing failures in untouched
  files, overnight seed matrix (`FLEET × [7,8,9]`).

## 6. Session 2026-09-17 — GREEN the RED: retention + duplicate identity (2 slow gates)

TDD, three slices back-to-back. Final state: **289 nextest green**,
all static gates green, **`rooms_rejoin` PASS (630s)**.

**Slice A — retention rule.** `sync_global.rs`: total = Σ labeled
slots (max with tombstone on overlap) + tombstones with local
provenance. Provenance = a locally-kept `seen` set of labeled ids
(`Local<HashSet>`; cleared when the map empties, which `leave_room`
already guarantees room-scoping): snapshot claims for never-labeled
ids never inflate the total — the `unlabeled_snapshot_drops` mesh test
stays green, and the `_4` iter-2 ghost family stays dead.

**Slice B — tombstones on the wire.** `SyncAck` replies now union
retained tombstones, max per id (the `publish_snapshot` pattern copied
into a `SyncCtx` SystemParam bundle + private
`sync_ctx_snapshot_entries.rs` delegate; `absorb_sync` drops 8→6
params). Provenance-gated the same way, so phantom zero-tombstones
(first-update snapshot chunks for never-live ids) can neither ride
replies nor derail labeling. `merge_entries` needed no changes
(unknown ids already park as absolute `PendingClick`s).

**TDD incident (scenario, not product).** The RED initially stayed RED
after the fix (`6 vs 23` unchanged): the turmoil leaver slept 30
virtual seconds *without pumping*, so its 17 clicks were queued but
never sent — it died silent and survivors rightly learned nothing. A
real process lives until killed; the leaver now pumps while waiting
(`total_retained.rs:34-53`). Lesson: a passing gate needs the scenario
to actually transmit the data under test.

**Slow gate 1 RED → duplicate player identity.** `agree_within` failed
after creator rejoin with `global=66` on one instance (16+16+17+17:
two live 16-slots — stale roster id + new peer id) and `p1=0` on
another (Moves arrived per `pos_log`, no roster slot to label). Fix,
two halves of one defect: `resolve_player.rs` never labels an already-
taken `PlayerNo` (`duplicate_player_no_never_labels_twice`); global
counts labeled slots only, unresolved merge artifacts join at labeling
time (`unlabeled_count_never_counts`). Overlap probe confirmed
live+tombstone never double-counts. Slow gate 2 **PASS (630s)**.

**Known next tail.** Rejoined peer ids still learn slowly on the
survivor side (roster freshness / dial-failed pruning, _5 §9) — masked
this run, expect it back under churn. Reply jitter + `Welcome.joining`
(slice 3), roster `Get` off the click path (slice 5), rig link-down
detection (§4b) all still open.
