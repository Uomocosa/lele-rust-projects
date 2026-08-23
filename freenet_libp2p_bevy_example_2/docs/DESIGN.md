# DESIGN

Two candidate architectures for `example_2`, mirroring the two framings in [[ARGUMENT]].
Both are recorded; **the choice is left open** until Phase 3.

[[README]] | [[FINDINGS]] | [[ARGUMENT]] | [[SKILL_PLAN]]

---

## System responsibilities (shared by both designs)

```
Bevy App (host)
  ├── freenet contract  ──► membership / authorization / persistent state  (trust anchor)
  └── libp2p swarm       ──► real-time transport: position, input            (relay)
```

- Contract = *who may join, what their identity/addrs look like, validity of state.*
- libp2p = *moving live game data once peers know each other.*
- Bridge = client reads the **validated** roster → derives a trusted `NetworkId` → dials
  peers over libp2p.

---

## Design A — Authority-in-contract (recommended)

### Contract (`contract/`)

Commutative-merge membership roster, hardened. **Final membership-gate structure**
(see [[ARGUMENT]] resolution):

```rust
// parameters (bincode) — public, stable, NOT secret; decides the contract instance
pub struct Params {
    pub namespace: [u8; 32],   // "exact app" gate: part of ContractKey
    pub max_members: u16,      // cap
}

pub struct PeerEntry {
    pub peer_id: String,
    pub addrs: Vec<String>,        // capped (MAX_ADDRS)
    pub seq: u64,                  // monotone per-peer Lamport counter, NOT wall-clock
    pub identity_key: Vec<u8>,     // this peer's public key = membership binding
    pub signature: Vec<u8>,        // signs (peer_id, addrs, seq)
}
pub type RosterState = BTreeMap<PlayerId, PeerEntry>;
```

**Rules (self-certifying membership, [[ARGUMENT]] resolution):**

- **New member `P`:** accepted only if `P.entry` is **self-signed** by `identity_key`; stored
  in `RosterState`. Any exact-app peer may self-join (no root/admitter key).
- **Existing member `P`:** accepted only if `new.seq > stored.seq` **and** signed by `P`'s
  stored `identity_key` (prevents impersonating / overwriting another peer's entry).
- **`validate_state` rejects:** > `max_members`, `addrs.len() > MAX_ADDRS`, malformed /
  missing / invalid signature shape, non-increasing `seq` within an entry timeline.
- Merge laws (commutative / associative / idempotent per-entry LWW) **preserved** — the rules
  are per-entry predicates, not global ordering.

**`seq` (Lamport) over wall-clock:** order-only, deterministic, no clock skew, no lockout.
Each peer is the sole writer of its own entry and self-increments; `update_state` just verifies
`new.seq > stored.seq`.

**`parameters` are public, not secret:** the contract will be open-sourced; `ContractKey` =
`Blake3(code_hash ‖ params)` already gates to the exact app + params. Any peer wanting to play
must run the exact published wasm (+ params) — a modified/rebuilt copy yields a different,
invisible contract. Stable distinct `namespace` per app avoids cross-talk ([FINDINGS] §2).

**Canonical `.wasm` (operational):** a rebuild changes `ContractKey` even for identical source
(Rust/wasm builds are not byte-reproducible) → peers in different "rooms". Ship **one
committed, canonical `.wasm`** embedded via `include_bytes!` so every client joins the same
contract.

**Out of scope (this crate):** input-truth anti-cheat (deterministic-sim peer verification,
referee). The contract guarantees identical *functions* to all users, not *honest inputs*;
anti-cheat is future work elsewhere.

Clients:

- Keys, signing, monotone `seq` counters.
- Distinct `namespace` per app-instance family → isolated logical contracts
  ([[FINDINGS]] §2).

### Client (`src/`)

Port the example_1 skeleton, trimmed and clean:

- `cli/` — `--identity-dir`, `--contract-params`, `--freenet-local`, `--freenet-gateway`.
- `freenet/` — decoupled WebSocket client (direct port from example_1).
- `roster/` — now consumes validated, signed state; emits trusted `NetworkId` membership.
- `p2p/` — libp2p real-time transport (position/input).
- Domain game module (e.g. `boxes/`) — physics, interpolation, UI, client-side only.

### Sequence

1. Node boots; client signs its own `PeerEntry`.
2. Client pushes signed entry as an update; contract validates + merges (order-independent).
3. Client reads validated roster via `Get`/`UpdateNotification`.
4. Client maps members → `NetworkId`s → dials over libp2p for real-time sync.

---

## Design B — Literal WASM relocation (state-only subset)

Not viable for "most of the app" (see [[ARGUMENT]] Framing B). The only defensible subset is
an **authoritative game-history ledger**: append-only, order-independent records of accepted
game events, validated by the contract.

- Client still owns the full game loop, physics, libp2p real-time, UI.
- Contract additionally records a canonical event history for later joiners / replay, subject
  to the same signed-append + cap constraints.
- Used only as an audit/catch-up layer, never as the real-time path.

This is a **superset** of Design A (Design A minus this ledger). Design is opened later only
if a persisted, authoritative history is actually required.

---

## TODO for Phase 3 (implementation)

- [x] Resolve Framing A vs B → **Framing A** ([[ARGUMENT]] decision log, resolution 2026-08-23).
- [x] Confirm freenet `parameters` → `ContractKey` derivation from source
      (`key.rs:200-203`, `code.rs:94-99`; recorded in [[ARGUMENT]] resolution).
- [ ] Confirm client-side signature scheme / delegate plumbing for signed updates
      ([[FINDINGS]] open thread 1: which key signs the entry, ed25519 encoding).
- [x] Decide ordering primitive → **Lamport `seq`** ([[ARGUMENT]] resolution).
- [ ] Author the hardened membership contract (self-certifying signatures, `seq` LWW,
      caps) + commutative/idempotent tests (mirroring example_1's three merge-law tests).
- [ ] Scaffold `src/`, port `cli/`/`freenet/`/`roster/`/`p2p/`, add the game module.
- [ ] Ship a single **canonical committed `.wasm`** (rebuild changes the key — do not
      rebuild per deployment).
- [ ] Verify with the standard build routine + `CARGO_TARGET_DIR=/tmp/frt-build`
      (space-in-path constraint) and `lele_lint`.