# Candidate 2 — Signatures (pubkey in `Parameters`)

Status: proposed — evaluate after Candidate 1
Contract: `GlobalCounterContract` (`contract/src/lib.rs:7`)
Client: `GlobalCounterClient` (`src/global_counter_client_method/tick.rs:38-46`, `connect.rs:30-35`)

## Goal
Make client forgery detectable *inside* the contract so invalid updates are no-ops for honest peers — logical isolation on the *same* `ContractKey`. Today any `BTreeMap{tag: value}` with larger `value` is accepted via `max`.

## Mechanism
- `Parameters` holds allow-list pubkey(s) (e.g., `Vec<[u8;32]>` ed25519). Part of `WrappedContract::new(code, params).key` (`connect.rs:31`), so allow-list changes fork `ContractKey` — flag day.
- Payload shape changes: `UpdateData::State` no longer bare `BTreeMap<u64,u64>` but envelope `SignedSlots { slots: BTreeMap<u64,u64>, sigs: BTreeMap<u64, Vec<u8>> }` or single-tag `Signed { tag, value, sig }`. `tick.rs` signs `bincode((tag, value))` with private key; contract verifies `ed25519_verify(pubkey[tag], msg, sig)` in `update_state:30-53` and silently drops failing tags.
- Tag↔pubkey binding: simplest is `tag = pubkey-derived u64` or explicit `BTreeMap<u64, Pubkey>` in `Parameters` checked in `validate_state`.
- `summarize_state` / `get_state_delta` unchanged (per-tag map).
- WASM embeds `ed25519-dalek` verify (`+30-100K` pre `opt-level=z`, still under Freenet size limit; verify `ls -lh contract/global_counter_contract.wasm`).

## How it stops cheating
On same key, attacker keeping canonical WASM but patching `tick.rs` to send inflated `value` without valid signature is dropped by honest peers' `update_state` — peers never `max`-merge it. Attacker only affects peers who also accept unsigned/invalidated updates (i.e., peers running a different contract that skips verification, which would have a different WASM hash → different key anyway). Honest network converges on signed values only.

## Harness gate
`run_suite.rs:5-18` still green if verification is pure (ed25519 verify is deterministic, no hidden reads). `update_idempotent` holds (same signed payload twice == once). `reads_data_not_state_plus1` holds (still reads from `data`). `validate_rejects_garbage` must now reject unsigned/garbage payloads as `InvalidUpdate` without panic. Need new `SuiteConfig` with valid `gen_update` producing signed data.

## Scaling
Still `O(users)` (one slot per user). Per-tag sig is constant overhead (64B). No `O(clicks)` history.

## Pros / Cons
Pros: strong anti-forge if private keys stay secret; no rate-limit tuning; contract-authoritative (logic in contract, your requirement).
Cons: key distribution & storage; `Parameters` rotation forks `ContractKey`; per-tag ownership model adds UX (who owns which `tag`); WASM size increase; attacker can still self-sign under *their own* tag and spam `+1`s (bounded only by Candidate 1 + rate); loss of key = loss of ability to increment.

## Open questions
- Single aggregator key vs per-user keys? Per-user is more decentralized but needs allow-list in `Parameters`.
- Privacy: pubkeys in `Parameters` are public on the `ContractKey` — acceptable?
- Payload migration: old bare `BTreeMap` updates become invalid → flag day; need versioned envelope.

## Verdict
Principled "all logic in contract" answer if cryptographic isolation on same key is required. Worth prototyping after Candidate 1 if you want to prevent unsigned forks from polluting honest peers' `max`-merge.
