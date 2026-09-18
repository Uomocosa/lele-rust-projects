# MAINNET PROBLEMS 7 — turmoil connection-problem coverage: what fast tests reproduce, what they don't, what's next

Follow-up to `MAINNET_PROBLEMS_6.md` (§6 retention + duplicate identity,
289-green gate) and `TURMOIL_TO_EXPLAIN.md` (plans (a) retention + (b)
link-down detection). This session implemented (b) in reduced form and
audited which real libp2p connection problems the turmoil rig can now
reproduce fast. **No `src/` changes — all work in `tests/mesh/`.**

## 0. What was built this session (all in `clicker/tests/mesh/`, uncommitted)

- **`total_retained_diverged.rs`** (new) — diverge→leave→heal agreement
  test: partition `peer-2` away at 15 virtual seconds, leaver `peer-3`
  clicks +11 extra at 20s (17→28, expected total `1+5+28=34`), leaver
  returns at 40s, repair + 10s grace, assert both survivors at 34.
  Went RED (`peer-2: 23 vs 34`, stale live 17, no re-ask) → GREEN.
- **`net_profile.rs`: `link_down_after`** — per-profile re-sync period
  (5s `lan`/`star-relay`/`leave`/`leave-relay`, 15s `flaky-nat`),
  threaded through `with_seed()`.
- **`turmoil_rig.rs`: rig-injected periodic re-sync.** Each `pump()`
  injects a direct `SyncReq` per outbound peer when
  `turmoil::elapsed() - last_sync > link_down_after` (`UpLink.last_sync`).
  Models the production 15s redial/heartbeat burst that cannot fire
  in-sim (`send_sync_heartbeat.rs:28` gates on Bevy wall-clock `Time`).
  Replies are tombstone-carrying `SyncAck`s (plan (a)), merges are
  idempotent max-merges — extra traffic is provably harmless.
- **Rejected: disconnect synthesis (v1).** Silence → roster prune +
  synthesized `PeerDisconnected`, first-envelope-back → `PeerConnected`.
  Deterministically froze every peer host at the first prune wave (~20
  virtual seconds) while the client sailed to timeout, and blew
  `total_retained` wall time 10s → 41–59s. Logical re-sync gives the
  needed healing without roster churn; socket-level kill deferred
  (would need a persistent listener + redial — partition already
  governs delivery, streams stay open so repair floods work).
- **TDD incidents worth remembering.** (1) Survivors recorded before
  the leaver left (`CONVERGE_ITERS` exhausted pre-`gone`); fixed with a
  `repaired` flag — record only after leave + repair + grace. (2) Extra
  clicks looked "lost" on the healthy link (`peer-1: 23`); actually
  early recording — healthy gossip propagates fine (`peer-1` reached
  28). (3) Partition-at-zero deadlocks `bring_up` (infinite dial retry,
  `"leaver never left"`) — see §2C. (4) The `51` inflation — see §3.

## 1. Coverage matrix: real connection problems vs the rig

Real-world inventory from `MAINNET_PROBLEMS.md`, `_5.md`, `_6.md`, `README.md`:

| # | Failure (production) | Turmoil today |
|---|---|---|
| 1 | Latency/loss variance, flaky NAT | ✅ per-pair `set_link_latency`/`set_link_fail_rate` + seeds |
| 2 | Stall (no traffic, links open) / heal burst | ✅ `partition`/`repair` |
| 3 | Message freeze + inspect | ✅ `hold`/`release` (`asymmetric_turmoil`) |
| 4 | Crash / process kill (leave) | ✅ host return → stream death → `PeerDisconnected` + prune |
| 5 | Star vs full-mesh topologies | ✅ `lanes_for()` (`star-relay`, `leave-relay`) |
| 6 | Post-heal re-sync (redial/heartbeat burst) | ✅ NEW: rig-injected periodic `SyncReq` (§0) |
| 7 | Gossip relay + dedup | ✅ `pump` dedup-forwards |
| 8 | Dial failure / timeout / `DialFailed` | ❌ `bring_up` retries forever (100ms, no deadline) |
| 9 | Stale roster / dead ephemeral ports | ❌ static `peer_key`, single `PORT`, roster pre-populated |
| 10 | Discovery (Freenet Get, PEX round, mDNS, gossip-mirror learning) | ❌ bypassed: `link_app` hardcodes roster + `PeerConnected`; `fixture` has no discovery task, no swarm |
| 11 | Duplicate sub-connections / exact link counting (P4 family) | ❌ exactly 1 stream per pair |
| 12 | Simultaneous-dial collision (lower-id-dials tie-break, desperation dial) | ❌ lanes dial one-directional patterns |
| 13 | NAT hairpin (same-IP dial fail) | ❌ no address model at all |
| 14 | Kad snapshot / position heartbeat periods | ❌ still Bevy wall-clock gated (fire ~once per sim); only `SyncReq` re-driven |
| 15 | Churn with new identity (leave → rejoin as new peer id) | ❌ hosts fixed at `sim.host()` time; `rejoin.rs` covers same-id only |
| 16 | Backpressure (`tcp_capacity` exhaustion) | ❌ messages too small/sparse to fill 1024 |
| 17 | Pre-labeling attribution race under churn | ❌ open — see §3 |

Slow-gate-only (explicit non-goals for fast tests): real Freenet `Get`
routing stalls, real NAT traversal, cross-OS sockets, real-Bevy-time
periodic paths (snapshots/positions), mDNS/PEX end-to-end, contract
replica splits.

## 2. Next functionalities (ordered, none started)

- **A. Dial failures + ghost pruning.** Dial deadline in `bring_up`
  surfacing a failed-dial outcome; seed one ghost entry; assert prune +
  live-only redial; cover the lower-id-dials tie-break + desperation
  dial (`README.md:49-52`). Pairs with the `_5.md:150-158 §9` product
  work (route `DialFailed` into discovery) — test first, product follows.
- **B. Seed matrix.** Parametrize leave/diverged cases over `[7,8,9]`
  (`star_turmoil` precedent); kills the `SEEDS` dead-code warning too.
  Cheapest confidence available (~3x case wall time).
- **C. Partition-during-handshake + dial deadline.** Partition at t≈0
  then `bring_up` must report failure instead of hanging the sim (the
  §0(3) deadlock, made representable). Enabler for A and §3-turmoil.
- **D. Duplicate connections / link counting.** Second stream per pair,
  close one mid-test, assert exact-once roster (P4 regression guard).
  Needs sub-connection identity in the envelope — medium invasiveness.
- **E. Snapshot/position heartbeat virtualization.** NOT recommended
  now: rig-invented snapshots assert state (vs `SyncReq`s which only
  ask) and could mask real bugs. Leave slow-only until a RED demands it.
- **F. Join handshake over loss.** `WantJoin`/`Welcome` → ready over the
  `flaky-nat` profile (`join.rs` is in-memory only today). Directly
  exercises the instant-join frontier (`Welcome.joining`, jitter,
  roster-`Get`-off-click-path).
- **G. This list.** Done — that is this file.

## 3. Closed: the `51` inflation (messenger-mismatch in drain fallback)

Observed mid-session (partition pre-convergence): `peer-1: 51 = 1 + 28
+ 22`, `stones={2:22, 3:0}`. Slice-1 repro (`tests/mesh/attribution.rs`,
deterministic, no turmoil) isolated the exact mechanism — pre-fix dump:

```
slots=[(Owner(1), Some(1), 0),
       (Owner(hash-peer-2), Some(3), 17),
       (Owner(hash-peer-3), Some(3), 17)]   // TWO slots labeled 3!
```

`find_slot` (`drain_pending.rs`) fell back to the *transport sender's*
slot when the logical owner was unresolvable — but for **absolute**
`SyncAck` entries the sender is just the messenger (a center peer
relaying third-party scores, `README.md` transitive sync). The absolute
`(3,17)` maxed the messenger's own unlabeled slot and **labeled it 3**,
while the true slot also reached 17 via sender-credits: 17 counted
twice, and both slots answer to 3 from then on
(`credit_sender` keeps crediting the mislabeled slot post-label).

Fix (product, one condition): sender-fallback applies to non-absolute
(delta) pendings only; absolute entries with no logical match stay
pending until the owner slot appears (same parking semantics as
`merge_entries`, retried every update). Guardrails kept green:
`drain_labels_slot_on_join`, `parked_absolute_drains_as_max` (logical
match path untouched), `sender_slot_pins_count`.

Collateral real bug found by the repro: `emit_flash` ∥
`despawn_on_leave` raced on cursor entities (parallel schedule, insert
on dying entity → Bevy command panic — the `_5.md:182` despawn-storm
family, now deterministic). Fixed by chaining
`(despawn_on_leave, emit_flash)` in `plugin_build.rs` and removing a
duplicate `emit_flash` registration (double-fire, pre-existing).

Regression tests: `attribution::unlabeled_clicks_park_exact`
(RED-proven: dual-`3` dump pre-fix) pins exact park `{2:5, 3:17}` +
retained 22; `relay_labels::prelabel_partition_no_inflation` (turmoil,
partition inside the unlabeled window) pins system-level exactness
`peer-1 == 34` (no inflation) and `peer-2 == 23` — the latter documents
a design limit, not a bug: `peer-2` never labeled the leaver and no
`Move`s can arrive after the leave, so absolutes stay parked per the
provenance rule (`unlabeled_count_never_counts`). Healing that case
needs periodic `Move`s (production heartbeat has them; the rig does
not — candidate follow-up, same injection pattern as the `SyncReq`
re-sync).

Residual, not traced to root: the exact `2:22` formation across hosts
likely needs multi-host snapshot feedback (cross-labeled entries
republished via `publish_snapshot` cycles) on top of the drain bug.
The creation vector is closed, so it cannot recur; reopen if seen.

## 4. Scoreboard

- Fast: **292 nextest green, 11 skipped** (was 290 — new attribution +
  relay cases); clippy/fmt/lint green. Mesh total ~19s.
- Slow gate (`freenet:run-rooms-rejoin`) NOT re-run — no `src/`
  changes, low risk, but remains the only true retention oracle.
- Next: §2B (minutes), then §2A (+§2C enabler), then §3 reproduction.
