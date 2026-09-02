# Candidate 1 — Bounded Increment (`value ≤ current + 1`)

Status: proposed — implement first
Contract: `GlobalCounterContract` (`contract/src/lib.rs:7`, `contract/Cargo.toml:2 global_counter_contract`)
Client: `GlobalCounterClient` (`src/global_counter_client.rs:12`, `src/global_counter_client_method/tick.rs:38-46`)

## Goal
Stop the trivial `MAX` cheat on the same `ContractKey`. Today `tick.rs:38-46` does `own = slots[tag]+1` and sends `BTreeMap{tag: own}` as `UpdateData::State`; `contract/src/lib.rs:46-48` does `*entry = max(old, incoming)` unconditionally. Attacker patching `tick.rs` to send `own = u64::MAX` converges all peers on inflated `count()` (`src/global_counter_client_method/count.rs:4 Σ`).

Forked WASM is already isolated via `ContractKey = Blake3(WASM || params)` (`src/global_counter_client_method/connect.rs:30-35`) — this candidate targets forked *client* only.

## Mechanism
In `update_state:30-53` per-tag:
```rust
let cur = *current.get(&tag).unwrap_or(&0);
if value > cur.saturating_add(1) { continue; } // or return InvalidUpdate
*current.entry(tag).or_insert(0) = (*current[&tag]).max(value);
```
Alternative: `return Err(ContractError::InvalidUpdate)` — either is idempotent if both orderings agree, but `continue` (silent no-op) avoids poisoning the whole batch when one tag is invalid.

`tick.rs` unchanged (`wrapping_add(1)`). `summarize_state:55-63` (per-tag map) and `get_state_delta:65-84` unchanged. `validate_state:22-28` still only `bincode` check.

## Harness gate
`freenet_contract_harness/src/run_suite.rs:5-18`:
- `validate_accepts_gen` / `rejects_garbage` — unchanged.
- `update_idempotent/communitative/associative` — preserved (max+bound is still idempotent; `value=5` applied twice == once).
- `reads_data_not_state_plus1:229-245` — preserved (still reads from `data`, just bounded).
- `summarize_detects_structural_divergence` — preserved (per-tag map).
- `delta_nonempty_and_roundtrips` / `handles_bad_summary` — preserved (delta still whole-tag map).
- `rejects_garbage_data_without_panic` — preserved.

Expect green without test changes.

## Scaling
`O(users)` (`BTreeMap<u64,u64>` one entry per tag). Delta is lagging tags only. No `O(clicks)` blow-up.

## Pros / Cons
Pros: cheapest, stops jump-to-MAX in one update, keeps CRDT pure, no key management, no WASM size change, no `ContractKey` fork.
Cons: cheater can spam many `+1` updates fast to outrun honest ticks. You explicitly accept this (`spam +1 ≠ cheat`).

## Open questions
- Policy for invalid value: silent drop vs `InvalidUpdate` error? Drop avoids failing honest batch with one bad tag; error is louder for debugging. Recommend drop.
- Should we also bound `value < cur` (replay) already handled by `max` — no-op, fine.
- `saturating_add(1)` vs `checked_add(1)` with overflow guard — `u64::MAX` edge.

## Verdict
Implement first. Fixes naive cheat, keeps harness green, keeps `O(users)`, no distribution change.
