# 13. Converge budget ≤500ms and slow score propagation — MITIGATED

Follow-up to: `MAINNET_PROBLEMS_9.md` §§2–3, `MAINNET_PROBLEMS_10.md` slice 5,
`MAINNET_PROBLEMS_11.md`, `MAINNET_PROBLEMS_13.md` run 4 (F5).
Related: `mainnet_problems/08-roster-get-stall.md`, `mainnet_problems/16-rejoin-reveal-log-flake.md`.

## Symptom

Post-drive room agreement must hold within **≤500ms** on same-host
runs — but measurements scattered 382ms → 731ms, and 1/5+ runs RED on
the budget alone.

## Evidence (logs)

Historic scatter (pre-fix): 731ms (`…155748`), 539ms (`…052230`),
526ms (`…053918`), 434ms, 382ms, 242ms, 148ms.

A/B on the same host sealed the cause. `tick_log` emits the `sync`
line only once per second; `agree_within` reads the **last** such line
every 50ms. The metric therefore tracks which instance crosses its
next 1 Hz log boundary last (offset per process start), not network
latency. Real propagation is 11–62ms (click gossip → `credit_click`).

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
- A/B same host: HEAD (1 Hz probe) = **485ms**; with the fix, 5 runs =
  **54 / 54 / 107 / 58 / 50 ms** — all ≪500ms, no budget ❌.
- Slow gate is still RED, but only on `rejoin-3`/`rejoin-1 a survivor
  logged the shared reveal` — a pre-existing flake (9/20 baseline) now
  tracked in `16-rejoin-reveal-log-flake.md`; both the HEAD A/B run and
  all fixed runs show it, so it is independent of this fix.

## Open / next slice

- Budget stays 500ms (user call); the probe fix makes it reliable.
- `label-from-SyncAck directly` was **attempted and reverted**: adding
  `ScoreEntry { owner, logical, count }` provenance + `PendingClick.logical`
  and labeling drained acks caused a creator-rejoin double-count
  (`instance-1-rejoin` `global=63` = 47+16, two slots labeled 1) and let
  ack labels bypass `reveal_on_join`. Reverted to keep the gate stable;
  a future slice must reuse `resolve_player`'s `taken` guard and the
  `JoinGate.pending` defer across every label site.
