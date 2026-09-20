# 14. Turmoil rig fidelity (connection layer) — LIVING DOC

Follow-up to: `TURMOIL_TO_EXPLAIN.md`, `MAINNET_PROBLEMS_6.md`,
`MAINNET_PROBLEMS_7.md`, `MAINNET_PROBLEMS_8.md`.
Related: every file above (the rig is how fast tests reproduce them).

## The 30-second model of turmoil

Turmoil (`turmoil = "0.7"`, `Cargo.toml`) runs N virtual hosts on one
OS thread: each host executes an `async` future with virtualized
TCP/clock/RNG. Seed 7 replays bit-identically. `partition` drops
messages but keeps streams open; `hold` freezes in-flight messages
for inspection; `crash` kills a runtime; per-pair
`set_link_latency`/`set_link_fail_rate` give each pair its own
characteristics. Our rig (`src/testing/turmoil/`,
`tests/turmoil/`): each host owns a real Bevy `App`
(`testing::fixture`); `pump()` = one `app.update()` + drain
`Commands` onto turmoil TCP + feed inbox into `Events`. Fleet
(`net_profile.rs`): `lan`, `star-relay`, `flaky-nat`, `leave` (+
`LEAVE_RELAY`); `rstest = "=0.27.0"` turns each into its own nextest
case.

## Coverage matrix (from `_7` §1, deltas from `_8` §3)

| # | Failure (production) | Rig status |
|---|---|---|
| 1 | Latency/loss variance, flaky NAT | ✅ per-pair latency/fail-rate + seeds |
| 2 | Stall / heal burst | ✅ `partition`/`repair` |
| 3 | Message freeze + inspect | ✅ `hold`/`release` |
| 4 | Crash / process kill (leave) | ✅ host return → stream death → `PeerDisconnected` + prune |
| 5 | Star vs full-mesh | ✅ `lanes_for()` / `plan_for.rs` |
| 6 | Post-heal re-sync burst | ✅ rig-injected periodic `SyncReq` (`UpLink.last_sync`, `link_down_after`: 5s calm / 15s `flaky-nat`) |
| 7 | Gossip relay + dedup | ✅ `pump` dedup-forwards |
| 8 | Dial failure / timeout / `DialFailed` | ✅ `dial_with_deadline.rs` (`dial_within` budget → `DialFailed` + `dial_failed`) |
| 9 | Stale roster / dead ports | ✅ `GhostHint` → `DialFailed` |
| 10 | Discovery (PEX/mDNS/gossip-mirror) | ◑ real dials + tie-break; PEX/mDNS still offline |
| 11 | Duplicate sub-connections / exact link counting | ❌ 1 stream per pair (P4 guard wants 2nd stream) |
| 12 | Simultaneous-dial collision / tie-break | ✅ higher-id-dials (`plan_for.rs`) + `redial_missing.rs` |
| 13 | NAT hairpin | ✅ `Nat::NoHairpin` (`nat.rs`, `dialable.rs`) |
| 14 | Kad snapshot / position heartbeat periods | ❌ Bevy wall-clock gated (fire ~once per sim) |
| 15 | Churn with new identity | ❌ hosts fixed at `sim.host()` time (`rejoin.rs` same-id only) |
| 16 | Backpressure (`tcp_capacity`) | ❌ messages too small to fill 1024 |
| 17 | Pre-labeling attribution race | ✅ closed via `attribution.rs` + `relay_labels` (`_7` §3) |

Slow-gate-only (explicit non-goals): real Freenet `Get` stalls, real
NAT traversal, cross-OS sockets, real-Bevy-time periodic paths,
mDNS/PEX end-to-end, contract replica splits.

## Rig lessons (don't re-learn)

- **Completion artifact vs real leave** (`_6` §1): a host future
  returning drops sockets like a crash — peers `park_until_done`
  after recording until the client releases them; only genuine
  leavers disconnect.
- **Listener = identity** (`_8` §2): `bring_up` dropping the listener
  after setup made late dialers fail non-deterministically — a
  persistent `accept_loop` task feeds `pump`; drain expected accepts
  before returning.
- **Lifecycle synthesis in `pump`**: stream death →
  `PeerDisconnected`, write failure → roster prune, gossip still
  dedups/forwards. Rejected: silence → synthesized disconnect (froze
  every host at the first prune wave, 10s → 41–59s wall); logical
  re-sync heals without roster churn.
- **Scenario fidelity before GREEN**: the RED that stayed RED after
  the fix (`6 vs 23`) was a leaver sleeping 30 virtual seconds
  without pumping — clicks queued, never sent. Real processes live
  until killed; leavers pump while waiting
  (`total_retained.rs:34-53`).
- **Partition-at-zero deadlocks `bring_up`** (infinite dial retry,
  `"leaver never left"`) — dial deadline (`_8` slice 1) made it
  representable.

## Next (from `_7` §2, remaining at time of writing)

- **D:** duplicate connections / link counting (2nd stream per pair).
- **F:** join handshake over loss (`WantJoin`/`Welcome` over
  `flaky-nat`).
- PEX/mDNS end-to-end over real streams (matrix #10's ◑).
- Seed matrix sweeps (`[7,8,9]` precedent) for new cases.

## Scoreboard (last recorded)

Fast: 325 nextest green, turmoil 17/17 green, mesh ~26s; clippy
`-D warnings`, fmt, `lele_lint`, `lele_bevy_lint`, `taxonomy_check`
green. `ui_test_plugin::tests::test_usage` fails only without
`DISPLAY` (pre-existing).
