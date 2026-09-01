# NETCODE

The lockstep protocol. **Option A: deterministic per-tick ordering** — every peer computes the
tick from the full set of inputs using one canonical order, over **real-time libp2p** transport —
with a **fixed command delay (input buffer)** so perceived latency is constant and never paced by
the slowest peer, and **commit-then-reveal** so nobody can react to another peer's same-tick input.

[[README]] | [[DIFFERENTIATION]] | [[ARCHITECTURE]] | [[CONTRACT]] | [[DETERMINISM]] | [[ANTI_CHEAT]] | [[ROADMAP]]

---

## Lockstep model, restated

The simulation never runs "free"; it advances in **ticks** (fixed timestep). The naive lockstep
waits for every peer's tick `N` input then advances — that makes one peer's bad network stall the
whole group. We do **not** do that. Instead every peer applies a **uniform fixed delay D** to its
inputs, so the group always has the full input set for a tick *before* it needs it.

```
input captured for tick N  ──(commit)──▶  reveal ──(wait D ticks)──▶  tick N applied
```

- **No peer-computed position is ever sent.** Only inputs and state hashes cross the wire.
- Perceived latency ≈ constant: `(D+1)·tick + a propagation hop` — independent of who's slowest,
  as long as all inputs arrive within the buffer.

## Option A — deterministic per-tick ordering (unchanged)

When the group applies tick `N`, each peer builds `ordered_inputs[N]` using a **canonical,
deterministic total order** (e.g. sort by identity `[u8;32]`, then `seq`):

```text
state_{N+1} = advance(state_N, sort(inputs[N], by = identity))
```

Every peer uses the same set and the same sort, so every engine produces the same `state_{N+1}`.
No single leader is required. This is still **real-time**: libp2p is the low-latency channel;
Option A only fixes the *application* order once the inputs are in.

## Fixed command delay (input buffer) — the latency fix

- Choose a **single global buffer depth `D`** (a few ticks; tuned to absorb typical jitter).
- Inputs whose effect should be seen at tick `N` are committed now and **applied at `N+D`**.
- When it is time to apply tick `N`, the group already has the complete input set — it gathered it
  earlier — so it never blocks on the *current* tick's in-flight delivery.

This is the classic "game lag / command buffer": latency is a **constant** fixed buffer, not the
random slowest-peer-per-tick. Networking stays real-time; the buffer just hides jitter.

## Commit-then-reveal — closes same-tick reactions

To stop a peer from seeing others' tick-`N` inputs and then picking its own to counter within the
same tick (the one genuine lockstep exploit):

```
per tick N:  peer commits  hash(input_N)   →  wait for all peers' commits
              →  each peer reveals  input_N  →  apply at N+D
```

- A peer cannot change its input after seeing the others' reveals, because its **commitment** was
  already broadcast. Revealing a different input fails the hash and flags the peer.
- Cost: **one extra propagation round per tick**, absorbed into the fixed buffer `D`
  (so `D` is sized to cover the commit + reveal rounds).

## Fairness & non-exploitability rules

| Rule | Guarantees |
|------|------------|
| **Uniform, fixed `D`** — one global constant, never per-peer, never extended for a slow peer. | A slow peer is disadvantaged (lag/idle), never favored. |
| **Cutoff + deterministic null-input.** Input for tick `N` must be committed & revealed by the cutoff at the start of `N+D`; if missing, that tick's input is a defined **null-input** and the peer is flagged late. | No per-tick stall; **no retroactive change** (tick `N` is fixed once applied). |
| **Liveness budget `B`.** A peer late for `B` consecutive ticks is treated as **offline**; the group continues without it (it rejoins later from the contract log, ROADMAP M3). | One bad link can't slow the game indefinitely; it just gets dropped. |
| **Fixed tick rate.** The sim advances no faster than the group's fixed tick + uniform `D`, regardless of any peer's network speed. | A fast peer can't speed the game up or pull ahead. |
| **Commit-before-reveal.** Ordering is commit-after-see-others'-commit, reveal-after-all-commits. | Withhold/switch inputs is impossible; reacting to same-tick inputs is closed. |
| **Withholding = self-harm.** Missing the cutoff → your input is idle and you're flagged. | Never an advantage to delay. |

## Tick protocol (draft)

1. **Announce intent** — session membership via the contract ([[CONTRACT]]).
2. **Commit** — each peer broadcasts `hash(input_N)` over libp2p.
3. **Reveal** — once all commits for `N` are seen, each peer broadcasts `input_N`.
4. **(buffer)** — the group does not apply `N` until `N+D`.
5. **Apply** — build `ordered_inputs[N]` (Option A sort) → `advance` → `state_{N+1}`.
6. **Hash + compare** — broadcast `hash(state_{N+1})`; each peer verifies it matches its own;
   mismatch flags divergence / a bad actor ([[ANTI_CHEAT]]).
7. **Commit to log (audit path)** — append the applied inputs for `N` to the contract's signed
   input log for the durable / catch-up record (async to the live path).

## Handling jitter / missed inputs / liveness

- **Within buffer:** normal jitter is absorbed; no visible latency changes.
- **Missed cutoff:** the peer's input for that tick is the deterministic **null-input**, flagged
  late — no stall.
- **Beyond liveness budget `B`:** the peer is treated as **offline** (its inputs become null), the
  group continues, and it must rejoin from the contract signed log ([[CONTRACT]]) before
  participating again.
- **Rejoin:** reconstruct history from the contract log, then resume.

## Session lifecycle

- **Join:** add self to the contract membership (self-certified) → learn member set + latest
  committed log.
- **Start:** the first member seeds the contract; the per-tick input set is over the current
  member set.
- **Leave/offline:** membership is monotone-additive here (contract doesn't remove entries); an
  offline peer simply yields null-inputs for later ticks and is excluded per the liveness budget.

## Networking stack (reuse from example_1/2)

- **libp2p** (TCP/QUIC + noise + yamux; `request-response` or stream) for commit/reveal/state-hash
  exchange — reuses the example_1/2 `p2p` layer nearly verbatim.
- The **identity** `[u8;32]` (ed25519 pubkey) is the stable sort key and the membership key, so
  identity, ordering, and membership stay consistent.

---

## Resolved (this-pass decisions)

- Latency model: **fixed command delay D** (constant lag, like other games), not slowest-per-tick.
- Fairness: **uniform global D**; slow peers are disadvantaged/excluded, never favored.
- Same-tick reactivity: **closed via commit-then-reveal** (+1 round absorbed into D).

## Open / to decide

- Concrete `D` and liveness budget `B` values (tuned in ROADMAP M1).
- Input coalescing: one input per tick vs aggregating multiple sub-frames.
- Whether excludes/rejoins reuse the offline-window budget from the example_2 automation logic.