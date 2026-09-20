# 04. Position publish stalls without mouse movement — SOLVED

Follow-up to: `MAINNET_PROBLEMS.md` Problem 4.

## Symptom

Two runs sat fully connected for ~5 min with zero player resolution,
then resolved within seconds of the first mouse move.

## Evidence (logs)

- Original campaign (run dirs deleted): `resolve_player` needs a
  gossip `Move`; `cursor resolved peer=` lines absent until first
  physical mouse move on each instance.

## Root cause

`publish_cursor` sent only on movement plus one startup burst (lost
before any peer connects). Idle mice = radio silence for minutes.

## Fix (with src:line)

5s position heartbeat in
`src/clicker/bevy_systems/publish_cursor.rs`; movement fast-path
unchanged (15 Hz). Heartbeat traffic is libp2p-local; zero Freenet
impact.

## Verification

- Resolution gate (`resolved_count >= 2` per log) passes every
  instance in `MAINNET_PROBLEMS_2.md` runs 4–6 (was 0–1 before).
- Kept green through all later campaigns.

## Open / next slice

None. Note for the rig: heartbeat still gates on Bevy wall-clock
`Time` (`send_sync_heartbeat.rs:28`, `publish_due.rs:9`), which
effectively never fires in turmoil sims — rig injects `SyncReq`
instead (`14-turmoil-rig-fidelity.md`).
