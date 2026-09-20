# 05. Game data plane is full-mesh-only — BY DESIGN

Follow-up to: `MAINNET_PROBLEMS.md` Problem 5.
Related: `mainnet_problems/11-score-retention-tombstones.md` (transitive sync closes the star for scores).

## Symptom

On a star topology (2↔3 edge missing) leaves never exchange clicks,
so exact-agreement checks fail even with a working mesh center.

## Evidence (logs)

- `MAINNET_PROBLEMS.md` Problem 5: `Click` and `SyncReq/SyncAck`
  travel by direct `Command::Send` to roster-known peers
  (`src/clicker/click.rs:20-27` at time of writing); gossip carries
  only positions; spawn/resolve require direct roster membership.
- `MAINNET_PROBLEMS_2.md` Finding 3: 3→1 click/pos flow continuously
  once connected; instance-1's counts of instance-2 stay 0 on the V
  topology (1-3-2).
- `MAINNET_PROBLEMS_2.md` Finding 4 (mitigation proof): 5s `SyncReq`
  heartbeats with full-entry `SyncAck` close the star for connected
  pairs — runs 3 and 5 show 2-of-3 exact agreement (87/87, 60/60).

## Root cause

Design, not bug: unicast game traffic needs direct edges; only sync
acks give transitive closure (within diameter rounds), and only for
connected pairs.

## Fix

None implemented — by design. The test must achieve full mesh
(resolution gate, `06-resolution-gate.md`). Deferred options if
partial meshes must work later: spawn cursor entities
opportunistically for gossip senders not in roster (with `LOBBY_CAP`
+ last-seen expiry), or route clicks over gossip.

## Verification

- Star/turmoil harness tests prove center-relayed gossip converges;
  transitive sync verified on mainnet (2-of-3 exact agreement).

## Open / next slice

Unchanged: keep the exact-agreement bar; do not relax the gate to
hide partial meshes.
