# 06. Test gate must assert resolution, not just connection — SOLVED

Follow-up to: `MAINNET_PROBLEMS.md` Problem 6.
Related: `mainnet_problems/13-converge-budget-500ms.md` (the bar got tighter later).

## Symptom

`connected + tick + ≥1 remote owner` passes on a star mesh minutes
before the game is actually synced (and an earlier revision asserted
tick-owner numbers that can never appear — ticks log advisory hashes,
not player numbers).

## Evidence (logs)

- Early campaign: gates passed while counts disagreed; exact
  agreement (`p1=15 p2=15 p3=16 global=46`, lag spread ≤2s) failed on
  every transient partition.

## Root cause

Connection ≠ attribution. Player resolution (`cursor resolved peer=`
lines) is the precondition for driving; the old gate did not assert it.

## Fix

Gate on `resolved_count >= 2` (`cursor resolved peer=` lines) in
every instance log, bounded by the 300s timeout. Later tightened to
per-peer counting proposals and the 500ms converge budget
(`13-converge-budget-500ms.md`).

## Verification

- Resolution gate passes every instance in
  `MAINNET_PROBLEMS_2.md` runs 4–6; kept through all later campaigns.
- `MAINNET_PROBLEMS_4.md` §4: final states `p1=16 p2=16 p3=16
  global=48` everywhere with the gate green.

## Open / next slice

Per-peer (not per-line) resolution counting (`MAINNET_PROBLEMS_3.md`
§4.4) — the line-count gate overcounts on label flapping
(`res=21` on instance-3, run `rooms-rejoin-090321`).
