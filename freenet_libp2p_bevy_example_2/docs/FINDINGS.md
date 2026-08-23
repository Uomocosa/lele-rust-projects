# FINDINGS

Research notes grounding the design question — "should `example_2` push more code into the
freenet wasm contract?" Sources: freenet.org build manual and the local crate sources.

[[README]] | [[ARGUMENT]] | [[DESIGN]] | [[SKILL_PLAN]]

---

## 1. What a freenet contract actually is

Read: https://freenet.org/build/manual/components/contracts and
https://freenet.org/build/manual/contract-interface

- Freenet is "a global decentralized key-value store where keys are WebAssembly code called
  Contracts." The **contract controls what state is permitted and how it can be modified**,
  plus how to synchronize state efficiently.
- A contract's state is "just a block of bytes." The serialization format is up to the
  contract (JSON, bincode, custom).
- Rust contracts implement `ContractInterface` (Layer 0), a low-level API with four pure
  functions:
  - `validate_state` — structural/semantic gate before a state is accepted.
  - `update_state` — merge inbound update data into current state.
  - `summarize_state` — produce a concise summary for sync.
  - `get_state_delta` — diff own state against a remote summary.
- `State`, `StateDelta`, and `Parameters` are all `[u8]` byte wrappers — no structured I/O,
  no secrets, no game loop. A contract runs as deterministic WASM on **untrusted peers**.

### Implication
A contract is a *pure state-transition + authorization function*. It cannot perform I/O,
hold secrets, run real-time streams, or drive a UI. Putting "an app" inside a contract in
the literal sense is bounded by these constraints.

## 2. `ContractKey` = code + parameters → namespacing

- `Parameters` "forms part of a contract along with the WebAssembly code." The contract key
  is derived from (code, params), so **distinct params produce distinct logical contracts.**
- Consequence for "other apps messing with our contract": two apps sharing the same WASM but
  using different `parameters` get separate contracts and cannot cross-talk. Shared
  `parameters` + shared key is the only way two apps collide on one contract — a real risk to
  document and design against.

Local references:

- `example_1/src/main.rs:47` — `params: cli.contract_params.map(String::into_bytes)` is passed
  to `roster::connect_and_run`, i.e. parameters are already a runtime input the app controls.

## 3. The signed-write pattern (the canonical abuse-resistance lever)

The manual's Blog example states verbatim:

> "The contract's code requires that new posts can only be added if they are signed by the
> blog's owner. The owner's public key is part of the contract's parameters."

This is exactly the mechanism described in [[ARGUMENT]] as "authority-in-contract": encode
**who may write** into `validate_state`/`update_state`, keyed on the owner public key baked
into `parameters`, and require signatures on update payloads. Adopting this turns the roster
contract from "accept any deserializable map" into "accept only signed, well-formed member
updates."

## 4. The commutative-monoid / CRDT constraint

- Contracts must merge any two valid states order-independently (eventual consistency,
  "similar to CRDTs"). Mathematically a commutative monoid on state.
- `update_state` must be **commutative, associative, idempotent** — it cannot encode
  sequential/step-based game logic, or rely on wall-clock arithmetic in a way that breaks
  order-independence.
- Local references confirm the current contract already obeys this:

  - `example_1/contract/src/lib.rs:32-38` — `merge_roster` unions keys, LWW per entry on
    `updated_at`.
  - `example_1/contract/src/lib.rs:141-247` — the three tests
    (`test_update_state_is_commutative/associative/idempotent`) lock this in.

- Consequence: any richer "game session" state must be designed as an order-independent
  merge (append-only logs, LC registers, LWW maps), not sequential transitions.

## 5. The current contract is NOT app-specific — and its validator is weak

`example_1/contract/src/lib.rs` defines a membership roster:

```rust
pub type RosterState = BTreeMap<PlayerId, PeerEntry>;
pub struct PeerEntry { pub peer_id: String, pub addrs: Vec<String>, pub updated_at: u64 }
```

- `validate_state` (lib.rs:51-59) only checks `bincode::deserialize::<RosterState>(...)`
  succeeds. It accepts **any** map that deserializes.
- Therefore any peer with access to the contract key can `Update` it with: id collisions,
  spoofed `peer_id`/`addrs`, inflated `updated_at`, or an unbounded `addrs` list. There is no
  authorization, no size cap, no monotonicity enforcement, no ownership check.

This is precisely the "other apps can mess with our contract" weakness that
[[ARGUMENT]] proposes to close.

## 6. Related-contracts validation (future lever)

- Cross-contract reads during validate/update are **not yet available**:
  https://freenet.org/build/manual/contract-interface references freenet-core issue #167.
- Notes it as future work that would let a contract validate against other contracts (e.g.,
  an identity or session contract). Design should not depend on it today.

## 7. Hybrid roles (from the frozen example_1 blueprint)

Read only; this slice is independent.

- `example_1/OBJECTIVE.md` — hybrid stack: freenet = identity/lobby/discovery + persistent
  state; libp2p = real-time position/input sync.
- `example_1/src/main.rs:28-72` — derives `own_id` from the libp2p keypair, spawns an
  embedded freenet node, and bridges roster events into Bevy.
- `example_1/src/p2p/derive_player_id.rs`, `example_1/src/p2p/peer_id_to_player_id.rs` —
  a `PeerId` is mapped (FNV-1a) onto a `u64` `PlayerId`, the roster key. This is a weak spot
  noted in [[ARGUMENT]]: a spoofed roster entry carries a spoofed `PlayerId`.

---

## Open research threads (for later passes)

1. Signature scheme availability on the freenet client side for signed updates (delegate /
   keypair plumbing) — needed for the signed-write design.
2. ~~Exact `ContractKey` derivation~~ **resolved (2026-08-23):** `ContractInstanceId =
   Blake3(code_hash ‖ params)`, `CodeHash = Blake3(wasm_bytes)` (`key.rs:200-203`,
   `code.rs:94-99`) — recorded in [[ARGUMENT]] resolution.
3. ~~Wall-clock vs Lamport~~ **resolved (2026-08-23):** Lamport `seq` chosen for LWW order, not
   wall-clock (deterministic, no clock skew) — [[ARGUMENT]] resolution.