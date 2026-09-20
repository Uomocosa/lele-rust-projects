# 08. Roster contract Get stalls on the click path — OPEN

Follow-up to: `MAINNET_PROBLEMS_5.md` §§6–9, `MAINNET_PROBLEMS_11.md`, `MAINNET_PROBLEMS_13.md` F4, `MAINNET_PROBLEMS_14.md`.
Related: `mainnet_problems/01-cold-start-partition.md`, `mainnet_problems/09-instant-join-path.md`.

Biggest gate killer (2/5 RED in the `_13` campaign). Freenet-layer
tail, known since `_5`. No fast test possible by construction — stays
a slow-gate-only row.

## Symptom

Same-shaped roster `Get` measures 336–622ms twice, then hangs past
two 5s kills. Creator never becomes ready; `join ready` never prints;
only `sync lobby=<room> p1=1 … players=1` ticks. Recurs run to run
(~1/3 of Gets observed hanging; 2/5 gates in `_13`).

## Evidence (logs)

- `_5` §6 (gates 3/4/6): `recv_after_get`
  (`src/discovery/recv_after_get.rs`) looped `recv_response()` with
  **no timeout**; node-side trace of a hung case: subscription ACK in
  ~50ms, state delivery never follows (routing/replication stall on a
  churned ring — NAT failures everywhere in the logs).
- `_13` run 2 (`rooms-rejoin-20260920-154557`, gate
  `/tmp/opencode/gate2.log`): `instance-1.log` `discovery: room
  resolved room-1789919174` (15:46:14) then `discovery: roster connect
  failed, retrying error=response timed out` at 15:46:24, 15:46:39,
  15:46:54, 15:47:09 — not one roster fetch in 60s+; `join abandoned:
  roster never resolved before the alone-cap` at 15:46:44.
  `sync-1.log` 0 bytes (joined nothing — settles the empty-file
  question).
- `_13` run 5 (`rooms-rejoin-20260920-160633`, gate
  `/tmp/opencode/gate5.log`): byte-identical shape — room resolved
  16:07:00-ish, roster `response timed out` ×4, `join abandoned` at
  16:07:20. F4 confirmed recurrent.
- `_7` §4 slow RED (`rooms-rejoin-20260918-191039`): room resolved via
  directory OK, libp2p sync flowing (`p1=1` 3.5s after click), but zero
  `expected`/Welcome/PEX lines — `JoinGate.expected` stayed `None`.
- `_11` (whole-run variant): stronger than `_9 §2` — instance-1
  `p3=0` for the **entire run**, `players=1,2,3` held for 33 sync
  lines, `p2=15` learned fine.
- `_14` §2: 4 attempts, 0 green — incl. `rooms-rejoin-20260919-180456`
  (dev binary, static-TRACE cap proven lifted) dying pre-drive with
  empty `sync-1.log`.

## Root cause

Per-node routing luck on a churned ring dominates: same key succeeds
30s earlier, then stalls. Subscription ACK fast, state never
delivered. The authoritative `Get` cannot meet 5s under churn.

## Fix (mitigations landed; structural slice open)

- 10s `CONNECT_TIMEOUT_SECS` (`src/discovery/constants.rs`) on the
  connect `Get` (`Error::ResponseTimeout` already existed).
- `switch_room` returns success; `drive_roster` retries across ticks
  via `pending_switch` (`src/discovery/run.rs`) instead of consuming
  the request on one failure.
- C1 dev profile (`Cargo.toml` `[profile.dev.package."*"]
  opt-level = 3`, `CLICKER_BUILD_PROFILE`, `freenet:run-rooms-rejoin-dev`
  task): proves decision-logging works, does not fix the stall.

## Verification

- Timeout bounds the hang but cannot fit a stalled `Get` into 5s.
- `_13` campaign: 2/5 RED with identical F4 signature.

## Open / next slice (the Q1 structural work, not started)

At switch, send `expected` = fresh hint-union (directory publishers +
room-tagged PEX + topic-tagged gossip — tagging gossip by room in
`drain_lobby_events` still TODO), dial immediately, PEX-confirm at
join, targeted syncs; contract `Get` union-upgrades in background;
prune dial-failed peers (route `DialFailed` into discovery first).
Bevy gate unchanged. Until then the 5s asserts stand as written;
expect tail reds on churned rings.
