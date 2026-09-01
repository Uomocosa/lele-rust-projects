# AGENTS.md — freenet_libp2p_bevy_example

> **CRUCIAL.** This project is the active development fork for **Phase C: live-join catch-up**
> (POLISH_2.md). Every change must pass the full verification gate before the next change.

## Iteration Protocol

Every change follows this cycle:

```
change → cargo build → cargo clippy → cargo fmt → cargo test → lele_lint → next change
```

**Never skip steps. Never batch unrelated changes. One logical change per commit.**

### Verification commands

```bash
cd freenet_libp2p_bevy_example
CARGO_TARGET_DIR=/tmp/frt-build cargo build --workspace --all-targets
CARGO_TARGET_DIR=/tmp/frt-build cargo clippy --workspace -- -D warnings
CARGO_TARGET_DIR=/tmp/frt-build cargo fmt -- --check
CARGO_TARGET_DIR=/tmp/frt-build cargo test --workspace --all-targets
cargo run --manifest-path ../lele_lint/Cargo.toml
```

> **Note:** `CARGO_TARGET_DIR=/tmp/frt-build` is mandatory — the workspace root contains spaces
> in the path, which breaks tikv-jemalloc-sys configure.

### Live mainnet check: run it EVERY iteration, not just at the end (MANDATORY)

The live mainnet automation is **not** a one-off final gate. Treat it as a **first-class
test** that runs on **every iteration** — every time we do a change and run the build/lint/
test cycle, we also run it as a check, exactly like a unit test.

Unit tests alone are not sufficient — the game runs on real Freenet mainnet instances and
convergence can only be proven end-to-end. Keep it in the loop so we catch regressions as
soon as they appear rather than at the very end.

```bash
cd freenet_libp2p_bevy_example
CARGO_TARGET_DIR=/tmp/frt-build cargo run -p mainnet-automation-4
```

Or use the slash command: `/run-local-mainnet ex4`

This builds the game, launches N real mainnet instances, waits for mutual convergence,
records a screen video, and sends the MP4 + report to Telegram.

**Failure is acceptable — it is a test.** A failed live run is NOT a blocker on its own;
like any test it is data. Report the outcome honestly (converged vs not, roster reach vs
`received peer input`, Put count / race, error signatures) and use the log to refine the
code. Iterate until repeated runs converge, but do not door-block an otherwise-correct
change on one flaky run.

**Keep it under control.** Because it launches real mainnet instances and records video,
never let uncontrolled instances or leftover game processes pile up. The automation's drop
guard kills all instances on exit; always confirm `pgrep` is clean afterward and that no
run dir (`.local-run/`) is left half-alive. One controlled run per iteration; never spam
runs back-to-back unattended.

## Work Queue

POLISH_2.md `docs/POLISH_2.md` is the work queue. Focus on **Phase C — Live-join catch-up**.

### Phase C Steps (change → test → change → test)

1. Add `RequestSnapshot {}` and `Snapshot { tick, bodies, participants, from }` message variants
   — `src/p2p/netcode_msg.rs`
2. Handle `RequestSnapshot` inbound: reply with authoritative `Snapshot`
   — `src/boxes/bevy_systems/netcode_tick.rs`
3. On first `PeerConnected` to established peer, send `RequestSnapshot`; receive & pick
   deterministic authority (lowest `PlayerId.from`)
   — `src/p2p/run.rs`, `src/p2p/event.rs`
4. Cross-check state hash against second peer before adopting
   — `src/boxes/bevy_systems/netcode_tick.rs`
5. Re-baseline: at snapshot tick T, all peers spawn late joiner's body at default position;
   joiner restores authoritative state
   — `src/boxes/bevy_systems/netcode_tick.rs`, `src/netcode/lockstep*.rs`
6. Set `applied_through = T`, participants = snapshot set + joiner
   — `src/netcode/lockstep*.rs`
7. Integration test: two peers converge, third joins mid-session, assert all three converge
   on identical state hashes
   — `integration_tests_4/`

## Context

- Forked from `freenet_libp2p_bevy_example_3` for Phase C work.
- Phase A (fresh-key deploy) and Phase B (anti-cheat docs) are inherited from example_3.
- The convergence root cause (libp2p request_response misuse) is already fixed.
- Phase C (live-join catch-up) is the primary work remaining.
