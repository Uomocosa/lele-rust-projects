# freenet_libp2p_bevy_example_3

**Deterministic lockstep netcode.** The follow-on to
`freenet_libp2p_bevy_example_2`, where the architecture changes so that *every client runs the
exact same physics and nobody computes their own position — only **inputs** cross the wire.*

## Status

**Docs only.** No code. This crate captures the design intent for a future implementation, in the
same doc-first style as example_2.

## The one-line difference from example_2

> In example_2 the contract is the enforced authority over **membership and state-rules**, while
> the game ran client-side with **client-computed positions**. In example_3 the *physics engine
> itself* becomes the shared authority — but it lives in **one hashed deterministic engine run by
> every client**, and the only thing that flows between peers is **inputs** (plus state hashes).
> Cheating therefore reduces to *choosing inputs*, which is explicitly accepted.

Read why the engine can't live inside the freenet contract itself in
[[docs/DIFFERENTIATION]], and how the pieces fit in [[docs/ARCHITECTURE]].

## Navigation (obsidian-style links)

- [[docs/DIFFERENTIATION]] — what's different from example_2, and the goals / non-goals.
- [[docs/PLAN]] — the complete handoff plan to build the final app.
- [[docs/ARCHITECTURE]] — the deterministic-lockstep layout and data flow.
- [[docs/CONTRACT]] — the contract's enforced role: membership + signed input log.
- [[docs/NETCODE]] — the lockstep protocol (Option A: deterministic per-tick ordering), hybrid libp2p transport.
- [[docs/DETERMINISM]] — determinism guarantees and cross-platform physics risks.
- [[docs/ANTI_CHEAT]] — what determinism + recompute/audit catch, and what it can't (input lying, accepted).
- [[docs/ROADMAP]] — phased plan (M0–M3).

## Reference (read-only context)

- `freenet_libp2p_bevy_example_2/` (+ its `docs/`) — the concluded membership/authority example
  this example builds on; its contract/report lessons are referenced throughout.
- `freenet_libp2p_bevy_example_1/` — the original hybrid blueprint.

## Design decisions baked in

| Decision | Choice |
|----------|--------|
| Input ordering per tick | **Option A: deterministic per-tick ordering** — all peers apply `sim_step(state, inputs[tick])` in one canonical order (e.g. sorted by identity); no single leader. |
| Latency model | **Fixed command delay (buffer)** — constant game-style lag, never paced by the slowest peer; a chronically slow peer is disadvantaged/excluded, never favored. |
| Same-tick fairness | **Commit-then-reveal** — inputs are hashed/committed before reveal, so nobody can react to another peer's same-tick input. |
| Transport | **Hybrid** — libp2p for real-time input/state; freenet contract as the signed audit ledger + catch-up. |
| Determinism level | fixed-step `f32` first, fixed-point as a later mitigation if divergence bites — **recorded open** in [[docs/DETERMINISM]]. |