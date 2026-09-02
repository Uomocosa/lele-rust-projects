# Candidate 4 — Hash-Chain / Sequential Ticket

Status: proposed — explore only combined with Candidate 2
Contract: `GlobalCounterContract` (`contract/src/lib.rs:7`)
Client: `src/global_counter_client_method/tick.rs:38-46`

## Goal
Force attacker to iterate sequentially rather than jump, without global PoW.

## Mechanism
- Per-tag chain: `value_0 = 0`, `value_n = hash(value_{n-1} || tag || ...)` or `value_n = value_{n-1} + 1` with chain proof `chain_n = hash(chain_{n-1} || tag || value_n)`. Payload carries `((tag, value_n, chain_n), prev)`. `update_state:30-53` verifies `chain_n == hash(prev_chain || tag || value_n)` and that `value_n == value_{n-1} + 1` (or `value_n > prev` bounded). Contract stores per-tag `((value, chain))` instead of bare `u64` — state shape changes from `BTreeMap<u64,u64>` to `BTreeMap<u64, (u64, Hash)>`.
- `tick.rs` must maintain local chain head per tag and include it in `UpdateData::State`.
- `summarize_state` / `get_state_delta` must hash per-tag chain head, not just `value`, to detect divergence (same `value` with different chain = different summary).

## How it stops cheating (weak standalone)
If verification requires chaining from genesis, attacker wanting `value = 1_000_000` must iterate `1M` hashes — non-trivial but still cheap if hash is fast (1M blake3 ~ ms). Without signatures (Candidate 2), attacker can still grind their own chain from `0` arbitrarily fast — chain alone does not bind to identity, so they can forge a fresh chain for their tag at will.

## Harness gate
- `update_idempotent` holds (same `(value, chain)` twice == once).
- `reads_data_not_state_plus1` risky — contract must read `value` from `data` but also check adjacency to stored chain; still reads from `data`, but now validates against stored head. Harness expects `update_state` not to do `state+1` — this is borderline. `summarize_detects_structural_divergence` needs update to include chain hash or equal totals with different chains mask divergence.
- Need `gen_update` to produce valid chain links or harness will reject.

## Scaling
- If storing full chain history: `O(clicks)` state blow-up — avoid.
- If storing only head `value + hash`: stays `O(users)` but loses ability to detect forked history (same head `value` with different lineage looks identical). Storing head only still forces sequential grinding but does not prevent parallel forged chains per tag.

## Pros / Cons
Pros: sequential cost without global PoW; `O(users)` if head-only; conceptually ties increments into a verifiable chain.
Cons: standalone does not prevent self-signed fast chain (needs Candidate 2 identity); complexity high; state shape change breaks existing `BTreeMap<u64,u64>` clients; harness and `merge_slots.rs` need rework; tuning hash cost same dilemma as PoW; replay of old valid link.

## Open questions
- Who defines genesis? Per `tag` zero? What about late joiners with no history?
- How to resync after missed chain links? Need full chain walk from genesis vs checkpoints.
- Combine with Candidate 2: chain anchored to pubkey gives identity + sequence — then Candidate 2 alone may be sufficient, chain adds little.

## Verdict
Explore only in combination with Candidate 2 (signature + sequential nonce) — alone it moves complexity to `O(clicks)` or head-only weakness with little anti-cheat gain over Candidate 1. Defer.
