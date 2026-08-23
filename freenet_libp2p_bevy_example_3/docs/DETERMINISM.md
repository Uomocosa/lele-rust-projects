# DETERMINISM

Deterministic lockstep only works if every peer's engine produces the **identical** `state_{N+1}`
from the **identical** `(state_N, ordered_inputs[N])`. This document pins the guarantees and the
honest risks.

[[README]] | [[DIFFERENTIATION]] | [[ARCHITECTURE]] | [[CONTRACT]] | [[NETCODE]] | [[ANTI_CHEAT]] | [[ROADMAP]]

---

## The determinism contract

`advance(state, inputs) -> state` must be **bit-for-bit reproducible** across machines, threads,
and (ideally) OSes for the same inputs. Three things must be identical on every peer:

1. **Engine code** — everyone runs the same hashed, deterministic build (see *Code identity*
   below).
2. **Simulation parameters** — fixed timestep, same constants, same world seed.
3. **Ordered input set** — the Option A canonical order ([[NETCODE]]): same member set, same
   inputs, same sort ⇒ same `ordered_inputs[N]`.

If any of these differ, state hashes diverge and the peers flag it ([[ANTI_CHEAT]]).

## Known divergence risks (be honest)

| Risk | Cause | Mitigation |
|------|-------|------------|
| **Cross-machine floating point** | `f32`/`f64` math can differ across CPUs/architectures (FMA contraction, extended precision, SIMD reordering). | Record as an open decision: start with fixed-step `f32` + a defined op order; if divergence bites, move to **fixed-point** math (integer) or a constrained numeric crate. Decide in M0. |
| **Non-deterministic constructs** | HashMap iteration order, `rand`, wall-clock, `SystemTime`, parallel reduction with floating accumulation. | Ban in the engine: use `BTreeMap`, seeded PRNG only (if any), no clock/time, no floats in ways that reorder. |
| **Build drift** | Same source rebuilt differently ⇒ different bytes ⇒ different engine ⇒ different app (see *Code identity*). | Pin one canonical engine artifact per release (analogous to the contract wasm). |
| **Input order ambiguity** | Two peers sort the same set differently ⇒ divergence. | Option A canonical sort by identity (`[u8;32]`), then `seq` — deterministic on the same set. |

## Code identity (the "same app" guarantee)

Two identities pin "this is my app":

- **Contract:** `ContractInstanceId = Blake3(code_hash ‖ params)`, `CodeHash = Blake3(wasm_bytes)`
  — a different/rebuilt contract is a different, invisible contract (freenet skill fact).
- **Engine:** the deterministic sim is distributed and **hashed** the same way. A peer running a
  patched engine either fails the state-hash comparison immediately (divergence) or is a
  different "app" that the group's audits reject ([[ANTI_CHEAT]]).

So "same exact rules for everyone" is enforced by: **contract membership** (you must be in my
contract) **+ engine hash** (you must run my exact engine) **+ hash comparison** (every divergent
step is caught).

## Fixed timestep

The engine steps at a fixed tick (e.g. 60 Hz), and inputs are consumed per tick. Rendering uses
interpolation so display can run at a different rate than the simulation clock — the *sim* is
never advanced by real elapsed time.

---

## Decision (recorded open)

- **M0 decision:** fixed-step `f32` with a defined numeric/operation order first; revisit with
  fixed-point if any cross-machine divergence appears in M0 determinism tests. Tracked in
  [[ROADMAP]].