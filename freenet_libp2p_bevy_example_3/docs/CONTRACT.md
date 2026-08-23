# CONTRACT

The freenet contract in example_3 is the **enforced, persistent layer** — but it is deliberately
*not* the simulator. It keeps and extends example_2's membership model and adds a **signed input
log** so peers can audit the live history and rejoin.

[[README]] | [[DIFFERENTIATION]] | [[ARCHITECTURE]] | [[NETCODE]] | [[DETERMINISM]] | [[ANTI_CHEAT]] | [[ROADMAP]]

---

## What the contract does (and the one thing it must NOT do)

**Does:**
- **Membership** — reuse example_2's model exactly:
  - Roster keyed by the member's ed25519 public key `[u8;32]`.
  - Self-certifying signed entries; a peer may only write its own entry.
  - Monotone per-peer `seq` (order-only counter); no rewind; caps (`max_members`, per-entry cap).
- **Signed input log** — a bounded, per-player, monotone-`seq` append of each peer's claimed
  inputs (the same inputs they broadcast live over libp2p). The contract validates:
  - form (well-typed input payload),
  - authentication (signed by the member's key),
  - monotonicity (`log_seq > stored_seq`), and caps (max rate / total length → ring buffer).

**Must NOT do (explicit non-goal):**
- Run `sim_step`. The contract never advances the simulation and never holds per-frame state.
- Be the source of the game rules — the deterministic engine is ([[ARCHITECTURE]]).

Why: everything the contract hosts is **commutative and order-independent** (per-player
appends + membership). Simulation is sequential and deliberately kept out (see
[[DIFFERENTIATION]]).

## Membership + input log combined

A natural combined state shape (design sketch, refine in implementation):

```text
RosterState = BTreeMap<[u8;32], Member>
Member {
  peer_id: String,
  addrs:   Vec<String>,                 // libp2p addrs, capped
  seq:     u64,                         // membership re-publish counter
  signature: Vec<u8>,
  // input log (bounded ring per player):
  input: {
    log_seq: u64,                       // last input appended
    tail:    Vec<HashedInput>,          // ring buffer of recent committed inputs
  },
}
```

- **Membership** updates are re-built exactly like example_2 (`PeerEntry` + signature).
- **Input log** appends are simply `+1` on `log_seq` and push `HashedInput` into the ring;
  both are commutative per-player (last-writer-wins register + append), so the merge laws hold.

## What the contract enforces vs the engine

| Property | Enforced by |
|----------|-------------|
| Who may join / write membership | contract (self-certifying) |
| Inputs are well-formed, signed, monotone | contract (input log) |
| What a box may be / physics / order of application | engine (deterministic, hashed) — **not** the contract |
| Are committed inputs honest (match live play) | peers recompute + audit ([[ANTI_CHEAT]]) |

## Catch-up / rejoin

A rejoining peer locates the contract (via its key, from the exact hashed engine + params build)
and pulls the signed input log to reconstruct the committed history before participating. This is
why the input log is valuable: it is the **auditable, network-enforced record** of what "really"
happened, even though the live path is libp2p.

## Contract identity note (carried from example_2 / freenet skill)

`ContractInstanceId = Blake3(code_hash ‖ params)`, `CodeHash = Blake3(wasm_bytes)`. A **rebuild
changes the key** even with identical source, so a game is a "room" pinned to one exact wasm +
params. Open-sourcing both is fine: a modified build is a different, invisible contract. For
deterministic lockstep this extends to the **engine build**: a patched engine is a different app
and will not be accepted by peers' audits ([[DETERMINISM]], [[ANTI_CHEAT]]).

---

## Scope notes

- The input log is bounded so `StateBytesWritten` / fan-out metering stay low (see
  [[DIFFERENTIATION]] metering point). Large unbounded histories would make the contract a
  cost-eviction candidate.
- Exact schema / ring size / rate cap are decided in [[ROADMAP]] M2.