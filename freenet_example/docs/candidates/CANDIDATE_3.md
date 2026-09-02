# Candidate 3 — Proof-of-Work Ticket

Status: proposed — deferred (O(clicks) concern)
Contract: `GlobalCounterContract` (`contract/src/lib.rs:7`)
Client: `src/global_counter_client_method/tick.rs:38-46`

## Goal
Throttle spammer without central authority or key management — make each increment cost CPU.

## Mechanism
- Payload becomes `PoWEnvelope { slots: BTreeMap<u64,u64>, nonce: u64 }` or per-tag `((tag, value, nonce), hash)`. `update_state:30-53` verifies `blake3(bincode((tag, value, nonce)))` has `N` leading zero bits before accepting via `max`. `tick.rs` must grind `nonce` until condition holds before sending `UpdateData::State`.
- `N` tuned per `ContractKey` (hard-coded in WASM or in `Parameters`). `validate_state` unchanged; `summarize_state`/`get_state_delta` unchanged.

## How it stops cheating (partially)
Honest tick pays `2^N` hashes once per second; attacker spamming many `+1`s pays `k * 2^N`. Jump to `MAX` in one update still requires PoW for that single value, but Candidate 1's bound is still needed to prevent one-shot inflation — PoW alone does not bound value, only cost per update.

## Harness gate
Verification is pure and deterministic, so `update_idempotent/communitative/associative` and `reads_data_not_state_plus1` remain green. `update_rejects_garbage_data_without_panic` must reject insufficient PoW as `InvalidUpdate` without panic. Need `gen_update` to produce valid PoW nonces or harness will reject all updates.

## Scaling
- `O(users)` if only latest `value` per tag stored (current G-counter). CPU cost is `O(clicks)` (grinding per click) — your stretch concern. If contract stored `nonce` history to prevent nonce reuse, state would grow `O(clicks)` — avoid. Without history, attacker can replay a valid PoW for same `value`.
- Tuning `N` is coarse: `N=10` ~1K hashes, `N=20` ~1M hashes — 1s tick with `N=20` may stall `tick.rs` loop (`main.rs:196-205` 1s sleep).

## Pros / Cons
Pros: no keys, no `ContractKey` fork on allow-list change, rate-limits spam economically.
Cons: CPU tax on honest 1s tick; `N` hard to tune (too low = cheap spam, too high = honest stall); moves problem from `O(users)` to `O(clicks)` CPU cost; still needs Candidate 1 bound for single-shot inflation; WASM hash cost.

## Open questions
- What `N` keeps 1s tick responsive on WASM clients (browser, low-end device)?
- Replay prevention without storing history? Accept replay if `value` already `max`? Then PoW can be reused for same `value`.
- Should PoW be per-tag or global? Per-tag is fairer.

## Verdict
Deferred per your note: already `O(users)` is a stretch, PoW pushes cost to `O(clicks)`. Consider only if key management (Candidate 2) is undesirable and spam `+1` rate becomes a real problem despite Candidate 1.
