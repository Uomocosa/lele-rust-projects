# CONTRACT_PLAN

The **example_2 membership contract** — full function inventory, lele-conformant atomic layout,
and test plan. Implemented per the [[ARGUMENT]] resolution (authority-in-contract, self-certifying
membership, Lamport `seq`, public params) and the [[DESIGN]] membership-gate.

[[README]] | [[FINDINGS]] | [[ARGUMENT]] | [[DESIGN]] | [[TESTING]] | [[SKILL_PLAN]]

---

## Roster model (final)

```rust
pub type RosterState = BTreeMap<[u8; 32], PeerEntry>;  // key = member's ed25519 public key

pub struct Params    { pub namespace: [u8; 32], pub max_members: u16 }
pub struct PeerEntry { pub peer_id: String, pub addrs: Vec<String>, pub seq: u64, pub signature: Vec<u8> }
```

- **Key by `identity_key`** (the member's ed25519 public key, `[u8;32]`) — globally unique,
  no collisions, no `PlayerId`/E018/`derive_more` in the contract.
- `identity_key` is the map **key**, not a field (entries are self-aware via their key).
- **Signature** = `ed25519_sign(privkey, bincode((peer_id, addrs, seq)))` — exactly the
  `(peer_id, addrs, seq)` canonical bytes, **no namespace prefix**.
- **`seq`** = monotone per-peer Lamport counter (order-only, no clock skew).

---

## Function inventory

### Types (atomic files)
| File | Item | lele |
|------|------|------|
| `params.rs` | `Params { namespace, max_members }` | 2 named fields (E018 OK) |
| `peer_entry.rs` | `PeerEntry { peer_id, addrs, seq, signature }` | 4 named fields (E018 OK) |
| `roster_state.rs` | `pub type RosterState = BTreeMap<[u8;32], PeerEntry>` | type alias (E001 N/A) |
| `error.rs` | `Error` thiserror: `InvalidParams, TooManyMembers, TooManyAddrs, SignatureInvalid, Rewind` | — |
| `constants.rs` | `MAX_ADDRS: usize` | exempt |

### Helpers (one pub fn file each + `test_usage`)
| File | Signature | Origin |
|------|-----------|--------|
| `decode_params.rs` | `decode_params(&[u8]) -> Result<Params, Error>` | new |
| `decode_update.rs` | `decode_update(UpdateData) -> Option<Vec<u8>>` | kept |
| `entry_bytes.rs` | `entry_signed_bytes(&PeerEntry) -> Vec<u8>` (`bincode((peer_id,addrs,seq))`) | new |
| `verify_entry.rs` | `verify_entry_signature(&[u8;32], &PeerEntry) -> Result<(), Error>` | new /* file `verify_entry_signature.rs` to satisfy E001 */ |
| `validate_entry.rs` | `validate_entry(&PeerEntry) -> Result<(), Error>` (`addrs.len() ≤ MAX_ADDRS`) | new |
| `validate_roster.rs` | `validate_roster(&RosterState, &Params) -> Result<(), Error>` (`len ≤ max_members` + per-entry) | new |
| `merge_entry.rs` | `merge_entry(existing: Option<&PeerEntry>, key: &[u8;32], &PeerEntry) -> Result<Option<PeerEntry>, Error>` | reworked (auth + `seq`) |
| `merge_roster.rs` | `merge_roster(&mut RosterState, RosterState) -> Result<(), Error>` | reworked (fallible) |

**`merge_entry` rules (self-certifying LWW):**
- `existing = None` (new member) → require `verify_entry_signature(key, incoming)`; insert.
- `existing = Some` → require `incoming.seq > existing.seq` **and** `verify_entry_signature(key, incoming)` over the **same** key; else `Rewind`/`Unauthorized`.

### `#[contract]` methods (lib.rs, delegating)
| Method | Behavior |
|--------|----------|
| `validate_state` | `decode_params` + `validate_roster` (caps + per-entry signature-shape) → `Valid`/`InvalidState` |
| `update_state` | decode current roster (default empty); for each decoded update `merge_roster`; any `Unauthorized`/`Rewind`/cap → `ContractError`; else `UpdateModification::valid(new)` |
| `summarize_state` | roster bytes (non-empty) |
| `get_state_delta` | full state (non-empty-delta guard) |

### Dropped / kept / added
- **Kept:** `decode_update`, non-empty-delta guard, merge-law tests.
- **Reworked:** `merge_entry` (auth + `seq`), `merge_roster` (fallible).
- **Added:** `decode_params`, `entry_signed_bytes`, `verify_entry_signature`, `validate_entry`, `validate_roster`, `Error`, `MAX_ADDRS`.
- **Removed:** `updated_at` → `seq`; owner-key gate → self-certifying.
- **Nothing useless in example_1's contract to drop** — each item maps forward.

---

## Atomic file layout

```
contract/src/
  lib.rs            #[contract] impl ContractInterface for RosterContract (delegates);
                    pub mod decls + re-exports; struct RosterContract;
  params.rs | peer_entry.rs | roster_state.rs | error.rs | constants.rs
  decode_params.rs | decode_update.rs | entry_bytes.rs | verify_entry.rs
  validate_entry.rs | validate_roster.rs | merge_entry.rs | merge_roster.rs
```

- `lib.rs`, `mod.rs`, `constants.rs` are exempt from E001/E006.
- One pub item per non-lib file, snake_case filename, `test_usage` each.
- E020: sibling files use `use crate::{params, ...};` (inside `use` only) or `super::`.
- No `Default`/constructors (no E013). No single-field structs (no E018).
- **No `.unwrap()`/`panic`** — use `?`/`map_err` (example_1 had one at `lib.rs:79`; avoid).

---

## Dependency

`ed25519-dalek = { version = "2", default-features = false, features = ["alloc", "std"] }` —
verify-only, pure-Rust. **Step 0 passed**: confirms it compiles for `wasm32-unknown-unknown`.

`bincode` + `serde` (already used by example_1) for the canonical bytes / params / entry.

---

## Status (implemented 2026-08-23)

- All 14 atomic files authored under `contract/src/`.
- Contract verified: `cargo test` (17 tests) ✅ · `cargo clippy -- -D warnings` ✅ · `cargo fmt -- --check` ✅ · `lele_lint` (exit 0) ✅ · wasm release build ✅.
- Canonical artifact committed at `contract/membership_contract.wasm`.
- **E001 fix:** `verify_entry.rs` → `verify_entry_signature.rs` (file named after its single pub item).
- **E011/E020 fix:** all cross-file imports are module-style (`use crate::...;` → `module::Item`), no inline `crate::` in submodules.
- Final `Error` variants trimmed to those actually used: `InvalidParams, TooManyMembers, TooManyAddrs, SignatureInvalid, Rewind`. `merge_entry` returns `Ok(None)` on equal-`seq` (idempotent no-op) and `Err(Rewind)` only on strictly smaller `seq`.

---

## Test plan

- **Per-file `test_usage`:** `decode_params` round-trip + reject bad/missing `max_members`;
  `entry_bytes` stable encoding; `verify_entry_signature` accept/reject; `validate_entry`
  over-`MAX_ADDRS`; `validate_roster` over-`max_members`; `merge_entry` new-self-signed-accepted /
  unsigned-rejected / rewind-rejected / wrong-signer-rejected; `merge_roster` union.
- **Contract-level (lib.rs):** merge-law suite **commutative / associative / idempotent** (now over
  `[u8;32]` keys) + negatives: non-owner rewrite, rewind, unsigned new member, over-caps.
  Mirror `example_1/contract/src/lib.rs` three merge-law tests.

---

## Verification

```bash
# wasm check (step 0)
CARGO_TARGET_DIR=/tmp/frt-build cargo build --release --target wasm32-unknown-unknown
# contract tests + lele
cargo test -p membership_contract -- --nocapture
cargo run --manifest-path ../lele_lint/Cargo.toml   # from contract/ and from crate root
```

- Lint from the **example_2 crate root** (no `src/` yet) via the new `--scan-folder` flag:
  `cargo run --manifest-path ../lele_lint/Cargo.toml -- --scan-folder=/contract`
  (relative to invocation, skips `target/`, one aggregate report). Scanned **clean (exit 0)**.
- The contract is E011/E020-compliant regardless of config (scanned with default config where
  `domain_import` is enabled).

---

### Decisions (locked)
- Roster keyed by `identity_key` (`[u8;32]`). — decided
- Contract structured as **atomic files**. — decided
- Signature over `(peer_id, addrs, seq)` canonical, **no namespace prefix**. — decided