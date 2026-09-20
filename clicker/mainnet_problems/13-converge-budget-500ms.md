# 13. Converge budget ≤500ms and slow score propagation — OPEN

Follow-up to: `MAINNET_PROBLEMS_9.md` §§2–3, `MAINNET_PROBLEMS_10.md` slice 5, `MAINNET_PROBLEMS_11.md`, `MAINNET_PROBLEMS_13.md` run 4 (F5).
Related: `mainnet_problems/08-roster-get-stall.md`.

## Symptom

Post-drive room agreement must hold within **≤500ms** on same-host
runs — but measurements scatter 382ms → 731ms, and two slow REDs show
a client missing all of one player's clicks (41s lag, then converged;
later: never, `p3=0` entire run).

## Evidence (logs)

- `_9` §2, run `.local-run/rooms-rejoin-20260919-113027/`: first
  `global=45` per log — `instance-3.log:971` 11:32:14.778,
  `instance-1.log:2150` 11:32:25.219 (+10.4s), `instance-2.log:2150`
  11:32:55.754 (+41.0s); meanwhile `instance-2.log:2113`
  11:32:54.740 `sync … p1=15 p2=15 p3=0 players=1,2 global=30`.
  Identity already resolved — score propagation lag, not roster.
  Passed only via 60s `agree_within` budget (~19s margin).
- `_11`, run `.local-run/rooms-rejoin-20260919-124436/` (332s): tails
  `instance-2.log` / `instance-3.log` `p1=15 p2=15 p3=15 global=45`
  vs `instance-1.log` `p1=15 p2=15 p3=0 players=1,2 global=30`.
  Instance-1 resolved `player=3` at 12:47:39 pre-drive, held
  `players=1,2,3` for 33 sync lines, learned `p2=15` fine — only p3
  entries never landed. `RUST_LOG=info` cannot distinguish
  no-`SyncAck`-arrived vs merge-dropped; fast repro must log the
  decision (later answered by `_12`: it was the fail-deadline kill).
- `_13` run 1 (`…154053`): `baseline-converge-ms: 434ms ≤ 500ms` ✅.
  Run 3 (`…155036`): `382ms` ✅. Run 4 (`…155748`, 476s): every
  functional check ✅, single ❌ `baseline-converge-ms: 731ms >
  500ms` → `phase_finish` (`tests/e2e_local/rooms_rejoin.rs:1360`)
  promotes the soft ❌ to fatal → RED on the budget alone. First
  `players=1,2,3 global=46`: instance-2 16:03:30.323, instance-3
  16:03:30.533, instance-1 16:03:30.565 (7× `p3=0 … global=46` then
  merge on the 8th — retention knew the total, the live slot lagged
  one SyncReq/heartbeat round-trip).
- `_9` §3 roster churn: `instance-2.log:553` alone `players=2` for
  ~14s; `instance-3.log` alone windows — expected around stops, worth
  watching.

## Root cause

App sync path only (layer rule): `send_sync_req`/heartbeat →
`SyncAck` carrying entries → `absorb_sync` merge into the labeled
slot. Late labeler waits one extra round-trip. (The never-merges
variant turned out to be the `_12` fail-deadline kill, now fixed.)

## Fix (measurement landed; hunt open)

- `agree_within` poll tightened 500ms→50ms; `phase_converge_drive`
  reports measured time-to-agree as `baseline-converge-ms` against
  `CONVERGE_BUDGET_MS = 500` (soft, run continues — but `phase_finish`
  promotes any soft ❌ to fatal, so effectively hard).
- Hunt restricted to the libp2p sync path (`send_sync_*`,
  `absorb_snapshot`, `apply_delta`).

## Verification

- 2/5 `_13` gates measured 434ms / 382ms ✅; 1/5 RED at 731ms.

## Open / next slice

- Candidate fast slices: label-from-`SyncAck` directly, or tighten
  the heartbeat trigger on post-drive growth.
- Alternative: relax budget to 1s — user's call (note the
  `phase_finish` promotion mechanics when setting the number).
- Investigate why instance-2 missed player 3's clicks for ~41s after
  `players=1,2,3` held (merge path: `send_sync_*`, `sync_global`,
  `publish_snapshot`, `absorb_snapshot`, `apply_delta`).
