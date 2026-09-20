# 02. Stale roster entries point at dead ports — MITIGATED

Follow-up to: `MAINNET_PROBLEMS.md` Problem 2.
Related: `mainnet_problems/09-instant-join-path.md`, `mainnet_problems/14-turmoil-rig-fidelity.md` (GhostHint).

## Symptom

`PlayerId` keys are stable (own_id 1/2/3) but libp2p ports are
ephemeral per run. A mature roster hands newcomers the previous run's
dead addrs; dials fast-fail (harmless) while live addrs propagate
slowly. An app trusting the roster blindly dials ghosts first.

## Evidence (logs)

- `MAINNET_PROBLEMS.md` Problem 2: mature-roster runs hand dead addrs;
  `dial_known` dials them (fast fail) — shape observation, no per-line
  log pointers recorded in the original file (run dirs deleted).
- `MAINNET_PROBLEMS_5.md` §7 (same family, cross-run): gate-3's
  "fresh" room `Get` returned 3 pre-existing slots — consecutive runs
  created identically-named rooms under the same namespace with
  default params → identical contract keys → ghost state from dead runs.
- Turmoil repro: `GhostHint` (previous-run address, closed port) →
  `DialFailed`, asserted pruned in
  `tests/turmoil/connect/dial_deadline_surfaces_failure.rs`.

## Root cause

Ephemeral ports outlive their usefulness in the contract; entries stay
"fresh" for `STALE_ENTRY_SECS` (300s), longer than any test run's
tolerance for dialing ghosts.

## Fix (with src:line)

- Burst announces at startup (immediate + every 5s for 15s, then 30s
  steady) in `src/discovery/run.rs`, so live addrs outrun stale ones
  within seconds.
- Stale entries (`updated_at` older than ~300s, `STALE_ENTRY_SECS`)
  are never dialed (`src/discovery/run.rs:publish_slots` /
  `absorb_roster`).
- Test-only: per-run unique `CLICKER_CONTRACT_PARAMS`
  (timestamp-micros + pid as even-length hex) set in-test before
  spawning, so each run gets fresh keys (`MAINNET_PROBLEMS_5.md` §7).
- Product follow-up (open): prune dial-failed peers — route
  `DialFailed` into discovery (currently only warned in
  `forward_events`, `src/main.rs:186-193`).

## Verification

- Fast: hint-store LWW/prune unit tests; turmoil ghost-prune case green.
- Slow: live addrs win within seconds on staggered runs
  (`MAINNET_PROBLEMS.md` 118s green run).

## Open / next slice

- `DialFailed` → discovery pruning (`MAINNET_PROBLEMS_5.md` §9, still
  TODO at time of writing).
- Contract change to per-(room,peer) directory entries to end
  single-entry LWW flapping (`MAINNET_PROBLEMS_3.md` §3.4).
