# DIFFERENTIATION

What example_3 does **differently** from example_2, and the goals / non-goals that shape it.

[[README]] | [[ARCHITECTURE]] | [[CONTRACT]] | [[NETCODE]] | [[DETERMINISM]] | [[ANTI_CHEAT]] | [[ROADMAP]]

---

## Where example_2 landed

`example_2` concluded with **authority-in-contract**:

- The freenet **contract** owns the membership/identity/authorization layer, enforced and
  un-bypassable: identity-keyed roster, self-certifying signed entries, monotone `seq`, caps.
- The **game** ran client-side: each client computed its own box position with its own physics,
  and exchanged position *snapshots* over libp2p.

That gives "if you use my app you abide by my rules" **for everything the contract enforces** —
but the client-computed physics meant the *actual movement* was not governed by the rules; only
the membership was. Two peers could both run the exact same contract and still diverge in
client-side behavior (e.g. one "speeds its box" by sending a legal-but-fake snapshot).

## What example_3 wants differently

The goal flips the game-facing part of example_2:

> **Every client runs the exact same physics engine, and no client computes its own position that
> it simply announces. Only inputs cross the wire.** The engine — not the client — determines the
> next state, and everyone's engine is byte-for-byte the same.

Concretely:

- **In example_2** the contract was the authority over *membership/state*, and positions were
  client-computed + exchanged as snapshots.
- **In example_3** the *game rules* become the shared authority too, via a single **deterministic
  sim engine** every client runs. Peers send only **inputs** (keys intended for tick *N*) and
  later compare the resulting **state hash**. No client-authored position exists.

## Why the physics does NOT live inside the freenet contract

This is a deliberate architectural shape, not a compromise. A freenet contract structurally
cannot be the authoritative simulator, for four reasons (grounded in freenet source + the
example_2 findings):

1. **No continuous tick.** A contract is a pure, on-demand `validate_state` / `update_state` /
   `summarize_state` / `get_state_delta` function. There is no per-frame loop ("the contract
   advances the sim") expressible in a contract.
2. **No global input order and no single authority.** Freenet state is replicated and merged
   **commutatively** (CRDT) with fire-and-forget updates and no consensus/ordering. A physics
   step is **sequential** — `state_{t+1} = sim_step(state_t, input)` — so it needs *one canonical
   `state_t` and one canonical input ordering*, which the commutative-merge model does not
   provide. Divergent orderings produce divergent physics the merge cannot reconcile.
   Non-commutative contracts are deprioritized / removed from the network.
3. **Metered and not a real-time channel.** Contracts are metered (`ExecCpuMicros`,
   `ExecFuelUnits`, `StateBytesWritten`, `BroadcastFanoutCost`, `BroadcastMessagesSent`) for
   cost-aware eviction. A 60 Hz sim in wasm per update would be an eviction-scale fan-out storm,
   and there is no low-latency contract→client channel.
4. **The client must render anyway.** Rendering, input capture, and interpolation are I/O on the
   client, by definition impossible in-wasm.

So the engine lives on every **client**; what the contract *retains* (membership + a signed
input log) is exactly what is commutative and order-independent. See [[CONTRACT]].

## Goals

- Everyone runs the **identical, deterministic** physics — the same exact rules, pinned to one
  hashed engine build.
- Only **inputs** flow between peers for the live simulation; nobody sends a "my position."
- Cheating reduces to **choosing inputs** — accepted (see [[ANTI_CHEAT]]).
- Reuse example_2's identity/membership + signed-input-log pattern for the enforced layer, and
  example_1/2's libp2p real-time transport for the live path.

## Non-goals (explicitly out of scope)

- Catching a player who presses buttons for another player / fakes the *source* of inputs
  (requires trusted hardware). Explicitly accepted as unavoidable.
- Hosting the physics sim inside the freenet contract (structurally impossible — above).
- Real-time sequencing with a trusted single authority; example_3 is symmetric p2p with
  deterministic ordering ([[NETCODE]] Option A).

---

## Decision log

| Date | Decision | Rationale |
|------|----------|-----------|
| _this-pass_ | Physics authority = shared client-side deterministic engine, not the contract | Contract can't tick / order / be authoritative (above). |
| _this-pass_ | Only inputs + state-hashes flow for the live sim | Engine is the sole position authority; nothing to spoof. |
| _this-pass_ | Cheating = choosing inputs; explicitly accepted | No trusted input source exists in p2p. |