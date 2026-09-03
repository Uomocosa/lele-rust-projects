# Candidate 6 — Fixed-shard accumulator O(S) (S=16)

Status: implemented — branch `candidate/c6-sharded`
Contract: `Shards=BTreeMap<ShardId,u64>` `NUM_SHARDS=16` `window=1` + shard sigs

## Nomenclature
- **Shard**: partition `shard = pubkey[0] % 16`. Fixed 16 buckets.
- **O(S)**: state size constant `S*8B` (~128B) vs `O(users)` linear.
- **Structural summary**: per-shard map, catches compensating divergence.

## Why strictly better on memory
`10k users`: `560KB → 128B` (4000×). Delta sends only lagging shards.

## Vulnerability
Shard collision Sybil: attacker grinds keys to same shard, inflates shard `+1` per msg same as honest. Mitigated by allow-list per shard (`Parameters` BTreeSet<Pubkey> per shard). Total-only guarantee documented.

## Harness
Per-shard `max` idempotent, `gen_divergent_equal_total` uses different shards.
