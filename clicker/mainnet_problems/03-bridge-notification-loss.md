# 03. Bridge drops the notifications it waits for — SOLVED

Follow-up to: `MAINNET_PROBLEMS.md` Problem 3.
Related: `mainnet_problems/01-cold-start-partition.md`.

## Symptom

`recv_response` skipped (discarded) `UpdateNotification`s while waiting
for a `SubscribeResponse` (up to 30s). Any foreign state arriving
mid-bridge was lost. Same hazard existed in the 10ms announce ack wait.

## Evidence (logs)

- Original campaign logs (`.local-run/clicker-<timestamp>/`,
  since deleted): foreign state arriving mid-bridge never merged;
  no per-line pointers were recorded — diagnosis was code-path
  inspection, confirmed by the fix's effect on subsequent runs.

## Root cause

Two readers / discard-on-wait: the bridge wait loop consumed only the
response type it wanted and dropped notifications arriving in-window.

## Fix (with src:line)

Single drain point (implemented): `poll()` is the only reader and
merges both `UpdateNotification` and `GetResponse` states; announce
and bridge are fire-and-forget (`src/discovery/run.rs`, bridge in
`src/discovery/bridge_tick.rs:attempt_reput`). Confirmations ignored.

## Verification

- Post-fix runs show bridge firings (14–16/run,
  `MAINNET_PROBLEMS_2.md` Finding 1) with no lost-notification
  signature; subsequent campaigns diagnose stalls at the `Get` layer
  instead (`08-roster-get-stall.md`), confirming this leg is closed.

## Open / next slice

None. Kept as regression reference: any new wait-loop on the
discovery task must drain through `poll()`, never `recv_response()`
directly.
