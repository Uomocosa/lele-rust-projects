# TURMOIL, EXPLAINED — what plans (a) and (b) do and why

Read this first if you are coming back cold. It explains the testing
rig, the bug it caught, and the two follow-up plans. All paths are
relative to `clicker/`.

## 0. The 30-second model of turmoil

Turmoil (`turmoil = "0.7"`, `Cargo.toml:90`) runs N virtual hosts on one
OS thread: each host executes an `async` future with virtualized
TCP/clock/RNG. No real sockets, no real time — `sleep` advances a
virtual clock, latency/loss come from a seeded RNG, so seed 7 replays
bit-identically. A `client` host drives chaos; `sim.run()` pumps until
the client completes:

```rust
let mut sim = turmoil::Builder::new()
    .rng_seed(7)
    .min_message_latency(Duration::from_millis(5))
    .simulation_duration(Duration::from_secs(600))
    .build();
sim.host("peer-1", || async move { /* a whole Bevy App lives here */ Ok(()) });
sim.client("driver", async move {
    turmoil::partition("peer-2", "peer-1");  // link-level: messages dropped (stall, NOT close)
    tokio::time::sleep(Duration::from_secs(10)).await; // virtual seconds, wall milliseconds
    turmoil::repair("peer-2", "peer-1");      // backlog floods, like a reconnect burst
    Ok(())
});
sim.run().expect("turmoil sim");
```

Semantics that matter (`turmoil-0.7.2/src/sim.rs`, `top.rs`):
`partition` drops messages but keeps streams open; `hold` freezes
in-flight messages for inspection via `Sim::links()`; `crash` kills a
runtime; per-pair `set_link_latency` / `set_link_fail_rate` give each
peer pair its own characteristics.

Our rig (`tests/mesh/turmoil_rig.rs`): each host owns a real Bevy `App`
(`testing::fixture`), and `pump()` = one `app.update()` + drain
`Commands` onto turmoil TCP + feed the inbox into `Events`. The fleet
(`tests/mesh/net_profile.rs`) describes networks as data — `lan`,
`star-relay`, `flaky-nat`, `leave` — and `rstest` turns each into its
own nextest test (`case_1_leave_lan`, `seed_1_7`, …).

## 1. The bug the rig caught (RED, on purpose)

`tests/mesh/total_retained.rs`: converge three peers to (1,5,17), let
peer-3's host return (socket teardown, like a process kill), settle:

```
assertion `left == right` failed: survivor peer-1 lost the leaver's total
  left: 6
  right: 23
```

Why, in three lines of cause and effect:

```rust
// 1. despawn_on_leave.rs:30-36 — leaver's entity despawns, score parked aside
tombstones.keep(**numbered, **counter);

// 2. sync_global.rs:9-15 — but the total only sums LIVE entities
for counter in &targets { total = total.saturating_add(**counter); }

// 3. resolve_player.rs:56 — the parked score is read exactly once, on rejoin
if let Some(saved) = ctx.tombstones.restore(*claimed)
```

So the total drops by the leaver's score until they rejoin. The parked
scores (`ScoreTombstones`, max-merge on keep) exist precisely so totals
could survive — nothing reads them for the sum. Your invariant
("room-bound, retained, deleted only with the room") already matches
the lifecycle: `leave_room.rs:12` clears tombstones on leave.

## 2. Plan (a): retention + tombstones on the wire

Two product changes, each RED-first:

**A1 — the sum reads tombstones** (`sync_global.rs`): total = Σ live
counters + Σ un-restored tombstones. Subtlety: a rejoined player can
have *both* (restore missed) — the sum takes max per id, uniform with
`keep()`/`merge_entries` semantics. Monotonic per room, cleared on
leave as today.

**A2 — `SyncAck` carries tombstones** (`absorb_sync.rs:68-82`). Today
the reply snapshots live labeled slots only, while the kad-snapshot
path already unions tombstones (`publish_snapshot.rs:44-59` — the
pattern to copy). Why A2 is not optional — the worked case:

```
A knows {1:1, 2:5, 3:17} → 23      B knows {1:1, 2:5} → 6     (B was partitioned from C)
C leaves. A tombstones 3→17, B tombstones nothing.
Naive A1 alone: A retains 23, B retains 6 — still disagreeing, e2e still red.
With A2: A's next SyncAck to B carries (3,17) → B's merge_entries parks/applies it → B heals to 23.
```

No new message types; `keep()` stays max-merge so replays are no-ops.
`Welcome.own_score` deliberately stays own-slot-only (tiny, never stale).
Green condition: `total_retained` 2/2 + full fast gates + slow gate
(`freenet:run-rooms-rejoin`) as the only "sure" oracle.

## 3. Plan (b): link-down detection in the rig (starts only on green)

What (a) cannot prove in-sim: after a `repair`, nothing re-asks —
`send_sync_heartbeat.rs:28` and `publish_due.rs:9` gate on Bevy
wall-clock `Time`, which advances by real microseconds per
`app.update()` while virtual time races, so periodic re-sync never
fires in turmoil. A diverge→leave→heal agreement test would fail even
with A2 — rig infidelity, not product bug.

(b) models TCP keepalive timeout: `UpLink` tracks per-origin
last-seen (virtual `turmoil::elapsed()`); silence beyond a
`LINK_DOWN_AFTER` threshold (a `NetworkProfile` field: ~5s calm, longer
for `flaky-nat`) synthesizes `PeerDisconnected` + roster prune, and any
envelope from a down peer synthesizes `PeerConnected` + re-dial — the
production reconnect-burst path. It unlocks `total_retained_diverged`:
partition B from C pre-leave, crash C, repair, assert both survivors at
23. It changes what every scenario observes, so it needs (a)'s green
baseline to regress against — all existing fleet cases must pass
unchanged.

## 4. Two rig lessons worth remembering

- **Completion artifact vs real leave** (§1 of `MAINNET_PROBLEMS_6.md`):
  a host future returning drops sockets exactly like a crash, so peers
  park after recording (`park_until_done`) until the client releases
  them; only genuine leavers disconnect.
- **Lifecycle synthesis lives in `pump`** (`turmoil_rig.rs`): stream
  death → `PeerDisconnected`, write failure → roster prune, gossip
  still dedups/forwards. `partition` correctly produces no events
  (stall ≠ leave).
