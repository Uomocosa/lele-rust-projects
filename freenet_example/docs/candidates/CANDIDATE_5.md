# Candidate 5 — Strict +1 G-counter (window=1)

Status: implemented — branch `candidate/c5-strict-plus1`
Contract: `contract/src/lib.rs:69 window=1` (was 5)
Base: G-counter `BTreeMap<Pubkey,u64>` max-merge + ed25519 + allow-list

## Goal
Reduce cheat from `+5` per message to `+1` per message with zero extra state.

## Mechanism
`update_state` per-tag `if value > cur.saturating_add(1) {continue}` (was +5). `tick.rs` unchanged.

## Why strictly better
Cheat `5→1` is 5× reduction. Honest lag `k` takes `k` RTTs vs 1 RTT before — at `1Hz` with `bridge_tick 30s Subscribe` heal indistinguishable.

## Nomenclature
- **Window**: max accepted `value - cur`. TTL/window concepts not needed here.

## Vulnerability delta
Spam `+1` at high rate still possible (P1 per OBJECTIVE.md, accepted). No new vuln. Burst offline `+k` needs `k` sends vs 1.

## Harness
Same as C1 with threshold `+1`.

## Scaling
Still `O(users)` exact.

## References
Atlas/River/Delta LWW `version` without window.
