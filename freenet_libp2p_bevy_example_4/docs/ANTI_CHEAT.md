# ANTI_CHEAT

An honest threat model for deterministic lockstep: what the design actually catches, what it
cannot, and what is explicitly accepted.

[[README]] | [[DIFFERENTIATION]] | [[ARCHITECTURE]] | [[CONTRACT]] | [[NETCODE]] | [[DETERMINISM]] | [[ROADMAP]]

---

## The core guarantee

**No position is ever authored by a client.** Positions come from the shared engine
(`state = advance(state, ordered_inputs)`), and every peer recomputes them from the same inputs
(Option A ordering, [[NETCODE]]). So you cannot "speed your box" by sending a different position
— there is no wire position to fabricate. Your box can only ever become what the shared engine
allows, given the inputs the group sees.

## What the design catches

| Attack | Caught by |
|--------|-----------|
| **Tampered / patched engine** | State-hash comparison per tick + engine hash identity. A divergent step is immediately visible; a different build is a different app ([[DETERMINISM]]). |
| **Injecting an invalid input payload** | The contract's signed input log validates form/auth/monotonicity ([[CONTRACT]]); live peers ignore inputs that fail the membership/`seq` checks. |
| **Forge another player's input** | Self-certifying signatures (contract); you can't sign for a member whose key you don't hold. |
| **Rewind / replay history** | Monotone per-player `seq` in the contract log; rewind is rejected. |
| **Same-tick reactive input** (see a peer's tick-`N` input, then pick your own to counter) | **Commit-then-reveal** in [[NETCODE]] — a hash commitment is broadcast before any reveal, so input can't change after seeing others'. |
| **Withholding inputs to delay/react** | Uniform fixed buffer + cutoff: missing the cutoff ⇒ your input for that tick is the deterministic null-input and you're flagged; never an advantage. |
| **Diverging from the committed history on rejoin** | A rejoining peer must reconstruct from the contract log and match the live hash; mismatch is detected. |

## What it cannot catch (explicitly accepted)

- **Input lying / autoplay.** A player (or another program) can send inputs that are "not what
  their hands feel like." The design cannot observe the source of inputs without trusted
  hardware. **This is accepted** — per the project's stated stance, cheating is reduced to
  *choosing inputs*, which is bounded and unavoidable in symmetric p2p. (Could later be
  mitigated only by a nominated authority / hardware attestation — out of scope.)
- **Side information / luck farming.** Choosing inputs optimally is not cheating by this model.

## How a caught divergence is handled

- On a state-hash mismatch, the offending peer's commits are flagged; it is excluded from
  further ticks (treated as offline) until it can rejoin from the contract log and match.
- The contract log is the durable, network-enforced record used for this ([[CONTRACT]]).

## Scope boundary

Deterministic lockstep + recompute/audit turns cheating into *input choice*, but does NOT try to
police *which* real buttons a human pressed. Everything beyond that (attesting the input source)
is explicitly out of scope for example_4.