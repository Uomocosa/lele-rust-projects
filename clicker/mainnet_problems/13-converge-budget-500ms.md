# 13. Converge budget ≤500ms and slow score propagation — MITIGATED

Follow-up to: `MAINNET_PROBLEMS_9.md` §§2–3, `MAINNET_PROBLEMS_10.md` slice 5,
`MAINNET_PROBLEMS_11.md`, `MAINNET_PROBLEMS_13.md` run 4 (F5).
Related: `mainnet_problems/08-roster-get-stall.md`, `mainnet_problems/16-rejoin-reveal-log-flake.md`.

## Symptom

Post-drive room agreement must hold within **≤500ms** on same-host
runs — but measurements scattered 382ms → 731ms, and 1/5+ runs RED on
the budget alone.

## Evidence (logs)

Pre-fix scatter, `baseline-converge-ms` summary line (all
`telegram_bot/.local-run/`):

- 731ms `rooms_rejoin-20260920-155746.log`
- 539ms `rooms_rejoin-20260921-052152.log`
- 526ms `rooms_rejoin-20260921-053916.log`
- 434ms `rooms_rejoin-20260920-154050.log`
- 388ms `rooms_rejoin-20260919-162337.log`
- 382ms `rooms_rejoin-20260920-155034.log`
- 251ms `rooms_rejoin-20260920-074643.log`
- 242ms `rooms_rejoin-20260921-052653.log`
- 148ms `rooms_rejoin-20260921-053436.log`

**Cause proof (A/B, same host).** The metric tracks each instance's
next 1 Hz `sync` boundary, not network latency. First
`sync … p1/p2/p3` final line per instance (`clicker/.local-run/`):

| build | run dir | instance-1 | instance-2 | instance-3 | spread | summary |
|---|---|---|---|---|---|---|
| fixed | `rooms-rejoin-20260921-083047` | `:2063` 08:32:14.311 | `:1279` .299 | `:1394` .279 | 32ms | 54ms |
| fixed | `rooms-rejoin-20260921-083501` | `:1593` 08:36:33.213 | `:1442` .213 | `:1180` .181 | 32ms | 54ms |
| fixed | `rooms-rejoin-20260921-083933` | `:3897` 08:44:13.418 | `:1393` .418 | `:1219` .385 | 33ms | 107ms |
| fixed | `rooms-rejoin-20260921-084618` | `:1442` 08:47:47.836 | `:1692` .823 | `:1084` .801 | 35ms | 58ms |
| fixed | `rooms-rejoin-20260921-085032` | `:1501` 08:51:58.543 | `:1254` .547 | `:1081` .527 | 20ms | 50ms |
| HEAD | `rooms-rejoin-20260921-085608` | `:2017` 08:57:56.756 | `:1228` .756 | `:1320` .057 | **699ms** | 485ms |

Fixed runs land within 20–35ms; the HEAD (1 Hz) run spreads
**699ms** (instance-3 `.057` → instance-1 `.756`), which is the probe
period. Summary files: `telegram_bot/.local-run/rooms_rejoin-20260921-{083013,083459,083930,084616,085029,085531}.log`.

## Root cause

`tick_log` (`src/clicker/bevy_systems/tick_log.rs:15`) gated the
`sync` line at `now - *last < 1.0`; `agree_within`
(`tests/e2e_local/rooms_rejoin.rs:630-645`) samples the last line at
50ms. A 500ms budget sits inside a 1000ms probe period, so the
metric is log-phase noise, not convergence time.

The historical "41s / `p3=0` entire run" variant is a **different**
path (parked `PendingClick` waiting for a `SyncReq`/`SyncAck` label
round-trip, plus the `_12` fail-deadline kill) — not the budget flake.

## Fix

- `tick_log` now emits the `sync` line **on state change as well as a
  1s heartbeat** (`src/clicker/bevy_systems/tick_log.rs`), keeping the
  `tick` per-owner lines at 1 Hz. The probe now resolves real
  convergence to one frame + poll.
- Fast repro: `tests/offline/converge/post_drive_agreement_ticks.rs`.

## Verification

- Fast: `lele:build/clippy/fmt/nextest/lint` green (380 tests).
- A/B: HEAD (1 Hz) 485ms vs fixed 54/54/107/58/50ms — all ≪500ms.
- Slow gate still RED, but only on the `rejoin-3`/`rejoin-1` reveal
  check — pre-existing, tracked in `16-rejoin-reveal-log-flake.md`
  (identical on HEAD and the fixed build).

## Attempts (not in git)

`label-from-SyncAck directly` (Phase 4A) was implemented and reverted
before commit. Design: new wire `ScoreEntry { owner, logical, count }`
provenance, `PendingClick.logical`, and draining acks to labels (a
duplicate logical on another slot is **dropped**; implementation style
extends `PendingClick`), plus a `JoinGate.pending` defer so ack labels
do not bypass `reveal_on_join`.

Run-by-run summary (`telegram_bot/.local-run/`): run1
`rooms_rejoin-20260921-080035.log` 48ms ✅; run2 `…080539.log` 92ms ✅;
run3 `…081239.log` 52ms ❌ (reveal check, see `16`); run4 `…081915.log`
73ms ✅; run5 `…082406.log` ❌ `phase rejoin creator: "rejoin-1: no
agreement"`.

Definitive regression, run5 (`clicker/.local-run/rooms-rejoin-20260921-082408/`):
`instance-1-rejoin.log:626-627` labeled two peers as player 1, then
`:629` reported `global=63`, while the survivors stayed
`instance-2.log:3626` / `instance-3-rejoin.log:1841` at `global=47` —
two slots labeled 1 double-count `47+16`. Reverted. A future slice
must route every label site through `resolve_player`'s `taken` guard
and the reveal defer before touching the ack path.

## Open / next slice

- Budget stays 500ms (user call); the probe fix makes it reliable.
- The real "late labeler" tail (parked pending → 5s `SyncReq` heartbeat)
  remains untouched; it only bites the historical 41s/never-merge case,
  not the budget.
