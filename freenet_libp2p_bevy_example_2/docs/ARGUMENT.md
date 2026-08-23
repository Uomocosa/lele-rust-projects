# ARGUMENT

The design question: given that `freenet_libp2p_bevy_example_1`'s contract is tiny and not
app-specific, should `example_2` **push more of the app into the wasm contract** so other
apps can't corrupt it?

This file documents **both framings** and a decision matrix. **The decision is deliberately
deferred** — it is resolved in Phase 3, right before implementation.

[[README]] | [[FINDINGS]] | [[DESIGN]] | [[SKILL_PLAN]]

---

## The two framings

Two distinct goals are nested under "move code into the contract":

1. **Make the contract app-specific / hard for others to corrupt** → a *validation +
   authorization* concern.
2. **Relocate as much application code as possible into WASM** → an *architecture
   relocation* concern.

They are easy to conflate, but they have very different answers.

---

## Framing A — Authority-in-contract (recommended)

**Thesis:** *don't move "most of the code" — move the **authority** into the contract.*

Ship the app's schema, invariants, and authorization as the contract, so membership is
enforced by the network rather than by trusting client code.

**What moves into the contract:**

- Member / identity schema (`PlayerId`, identity key format).
- `PeerEntry` shape: `peer_id`, `addrs` (capped), a **monotone Lamport `seq`**, `identity_key`.
- Authorization: **self-certifying** signed writes — each member signs its own entry; a member
  may only write its own `seq`-advancing entry. (Mesh with the manual's signed-write pattern —
  [[FINDINGS]] §3; finalized in the resolution below.)
- Size / pruning caps and structural rejection in `validate_state`.
- Optional order-independent "session / room" membership via a commutative merge.

**What stays client-side:**

- Bevy game loop, physics, interpolation, UI.
- libp2p real-time positional/input sync.
- Keypair storage, embedded node bootstrap, WS client plumbing.

**The bridge (hybrid):**

- Client reads the validated roster → derives a trusted `NetworkId` per member → dials peers
  over libp2p. The contract is the **trust anchor for membership**; libp2p is the **transport
  for real-time**. This preserves the proven example_1 split.

**Why this framing wins:**

- Respects the contract's hard limits (no I/O, no secrets, no game loop, deterministic,
  order-independent) — see [[FINDINGS]] §1, §4.
- Closes the actual weakness: `validate_state` today only checks deserializability
  ([[FINDINGS]] §5), so any peer can inject junk. Signed, monotone, capped writes stop that
  at the network layer.
- Directly serves the goal "peers with the same app connect to each other": *same params +
  valid signed roster = mutually trusted membership.*

**Costs / cautions:**

- Requires signature/counter plumbing client-side (self-signing key, monotone `seq`).
- Validation changes recompile and re-publish the contract (upgrade path exists; must be
  documented).
- Determinism forbids wall-clock authorization inside the contract; monotone LWW must come
  from signed payload fields or a Lamport timestamp.

---

## Framing B — Literal "most code in WASM"

**Thesis:** *push as much application logic as possible into the contract.*

**Why it fails fast (documented hard limits):**

- A contract is a pure, deterministic state-transition function running on **untrusted
  peers** ([[FINDINGS]] §1). No I/O, no secrets, no game loop, no real-time streams, no UI.
- The commutative-monoid/CRDT constraint ([[FINDINGS]] §4) **bans sequential/step-based game
  logic** and wall-clock-dependent transitions.
- You cannot run Bevy, avian2d physics, or libp2p snapshot sync inside a contract.

**What it still cannot achieve:**

- Real-time game state — belongs to libp2p, not the contract.
- Private state / secrets — belongs to a delegate, not a public contract.
- Interactive UI — client-side.

**Verdict:** viable only for a narrow, intentionally state-only subset (e.g. an authoritative
game *history* ledger), not for "most of the app." Documented here so the trade is explicit,
but it will not scale to the fruit of the stated goal on its own.

---

## Decision matrix

| Criterion | A: Authority-in-contract | B: Literal WASM relocation |
|-----------|--------------------------|----------------------------|
| Stops others corrupting our membership | ✅ signed+capped+validated | Partial (validation only) |
| Respects contract determinism / CRDT | ✅ | ❌ for sequential logic |
| Per-goal: same-app peers connect | ✅ directly | Partial |
| Feasible scope today | ✅ | ❌ (blocked by no-I/O, no-loop) |
| Maintainability | ✅ (thin contract, client logic unchanged) | ❌ (recompile/publish on every change) |

**Current lean:** **A**, with B allowed only for an optional authoritative game-history
ledger.

---

## Decision log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-08-23 | **Framing A (authority-in-contract)** | Final argument resolves Framing B; see resolution below. |

### Resolution (2026-08-23) — Framing A, "abide by my rules" = identical functions

The design goal is: **anyone may download the contract, but if they use it they can only use
the functions I publish.**

**How it's delivered — content-addressing + mandatory validation.** `ContractInstanceId =
Blake3(code_hash ‖ params)`, `CodeHash = Blake3(wasm_bytes)` (freenet-stdlib `key.rs`,
`code.rs`). A peer who wants to play with others must run the same contract instance = the
identical wasm bytes + params = **your** `validate_state`/`update_state`. A modified or
rebuilt copy yields a different key → a different, invisible contract → excluded. This is the
**identical-functions** guarantee: every user abides by the same rules, and open-sourcing the
contract strengthens auditability without weakening the gate.

**The identity-vs-input-truth cut.** Content-addressing guarantees everyone runs the same
*functions*; it cannot guarantee honest *inputs*. Physics is `outcome = physics(inputs, state)`;
inputs originate inside each peer's unobservable client, and the contract is a **gate over
proposed state**, not an authority over actual state. A cheater feeds the same shared physics a
fabricated input and gets the cheated outcome accepted. So "physics in the wasm" yields
*identical and auditable physics*, not *input policing*.

**Scope decision:** input-truth anti-cheat (deterministic-sim peer verification, referee)
is **deferred and explicitly out of scope for this crate**.

**Locked design elements:**
- Commutative/associative/idempotent per-entry LWW merge (CRDT), preserved.
- **Self-certifying membership** — a member may write only its own entry, signed by its own
  `identity_key`. No root/admitter key (any exact-app peer self-joins; impersonation of a
  specific peer is rejected).
- **Lamport `seq`** (monotone per-peer counter) for LWW order, not wall-clock (no clock skew,
  no lockout, deterministic).
- **Public, non-secret `parameters`** (namespace + policy). Open-sourcing makes secrecy moot;
  params are a stable public namespace, not a capability secret.
- Caps: `max_members`, `MAX_ADDRS` enforced in `validate_state`.

**Source:**
- `ContractInstanceId = Blake3(code_hash ‖ parameters)`: `freenet-stdlib` `key.rs:200-203`.
- `CodeHash = Blake3(wasm_bytes)`: `freenet-stdlib` `code.rs:94-99`.
- Contract receives no caller identity: `ContractInterface` `trait_def.rs:71-98`;
  `UpdateData` `update.rs:199-225` (only `related_to: ContractInstanceId`, no signer/sender).
  Self-signature in the payload is therefore required for any identity binding.

Framing A is now authoritative for `example_2`'s Phase 3 implementation; Framing B survives only
as the optional authoritative-history-ledger subset noted in [[DESIGN]].