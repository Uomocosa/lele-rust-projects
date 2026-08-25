# ROADMAP

Phased plan to implement example_4's deterministic-lockstep netcode (currently docs-only; the
expected future engine). Each milestone has acceptance criteria and a verification approach
patterned on the example_2 harness.

[[README]] | [[DIFFERENTIATION]] | [[ARCHITECTURE]] | [[CONTRACT]] | [[NETCODE]] | [[DETERMINISM]] | [[ANTI_CHEAT]]

---

## M0 — Deterministic sim engine

Goal: a pure, reproducible `advance(state, inputs) -> state` that is bit-for-bit stable over
repeated runs, across machines.

- Choose the numeric strategy ([[DETERMINISM]]): fixed-step `f32` with a defined op order first;
  switch to fixed-point if any cross-machine divergence appears.
- Ban non-determinism: `BTreeMap`, seeded PRNG (only if needed), no wall-clock, no float
  reordering.
- Engine hidden as a small library the clients share.
- **Acceptance:** determinism test runs the same input trace twice (in-process, then across two
  temp processes) and asserts identical final state; a golden trace is committed so a divergence
  is caught by CI. Uses the example_2-style test harness.

## M1 — Lockstep input/state sync over libp2p

Goal: N peers exchange inputs per tick and converge to identical state hashes in real time.

- Port the example_1/2 `p2p` layer (libp2p transport + direct connections) to carry per-tick
  inputs and state hashes.
- Implement the tick protocol + Option A canonical ordering ([[NETCODE]]): commit → reveal →
  sort → advance → hash → compare, under the **fixed command delay D** and **commit-then-reveal**.
- Tune the buffer depth `D` and liveness budget `B` (open in [[NETCODE]]); confirm constant
  perceived latency under injected jitter.
- **Acceptance:** a 2-node hermetic test (patterned on
  `integration_tests_2/local_two_node_production_sync`) shows both peers reach the same state
  hash after K ticks; a latency-budget check confirms per-tick time is **constant** (not paced by
  the slowest) within the buffer, and that a peer whose latency exceeds `D` becomes idle/excluded
  rather than stalling the group.

## M2 — Contract: membership + signed input log

Goal: the enforced, durable record that supports audit and catch-up.

- Reuse example_2's membership contract (identity-keyed, self-certifying, monotone `seq`, caps).
- Add the bounded per-player **signed input log** ([[CONTRACT]]): validates form/auth/monotonicity,
  ring buffer, caps. Keep merge commutative & metering low.
- **Acceptance:** contract unit tests (merge-law suite + negatives: unsigned input, rewind,
  over-cap) — mirror example_2 `contract/src` patterns; a hermetic two-node test commits inputs
  to the log and re-reads them.

## M3 — Audit / recompute / rejoin + session polish

Goal: self-protection and graceful group dynamics.

- On state-hash mismatch: flag + exclude the offending peer ([[ANTI_CHEAT]]); attempted rejoin must
  reconstruct from the contract log and match.
- Session lifecycle: join via membership; rejoin from signed log; offline/leave handling via the
  **liveness budget** from [[NETCODE]] (buffer + exclusion, resolved here).
- **Acceptance:** an audit test injects a tampered peer (wrong reveal) and asserts it is
  flagged/excluded; a rejoin test drops a peer for N ticks then rejoins it and asserts convergence
  against the contract log; a fairness test confirms the slowest peer is never extended a buffer
  (it becomes idle and is excluded, not favored).

## Verification (recurring)

Use the example_2 toolkit as reference:
- `cargo test --workspace --all-targets` + clippy `-D warnings` + `cargo fmt -- --check` +
  `lele_lint --scan-folder` for each crate; `CARGO_TARGET_DIR=/tmp/frt-build` for the
  space-in-path workspace.
- The `mainnet_automation`-style driver can later drive N real engine builds on the mainnet and
  send a screen video + report to Telegram (example_2 did this for membership; example_4 would
  drive lockstep convergence the same way).

## Follow-on (explicitly out of the core milestones)

- Deterministic re-simulation as a security hardening (peers re-run a bounded window over the
  contract log) if M3 audit shows gaps.
- Input-source attestation (trusted hardware / nominated authority) — explicitly scoped out of
  example_4 ([[DIFFERENTIATION]] non-goals).