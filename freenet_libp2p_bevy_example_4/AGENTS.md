# AGENTS.md — freenet_libp2p_bevy_example_4

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
cd freenet_libp2p_bevy_example_4
CARGO_TARGET_DIR=/tmp/frt-build cargo build --workspace --all-targets
CARGO_TARGET_DIR=/tmp/frt-build cargo clippy --workspace -- -D warnings
CARGO_TARGET_DIR=/tmp/frt-build cargo fmt -- --check
CARGO_TARGET_DIR=/tmp/frt-build cargo test --workspace --all-targets
cargo run --manifest-path ../lele_lint/Cargo.toml
```

> **Note:** `CARGO_TARGET_DIR=/tmp/frt-build` is mandatory — the workspace root contains spaces
> in the path, which breaks tikv-jemalloc-sys configure.

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

## Local Mainnet Test

Use `/run-local-mainnet` with target `example4` (or `example_4`, `ex4`):

```
/run-local-mainnet ex4
/run-local-mainnet ex4 3 release
```

This invokes the `mainnet-automation-4` crate which builds the game, launches N real
mainnet instances, waits for convergence, and sends a video + report to Telegram.

## Context

- Forked from `freenet_libp2p_bevy_example_3` for Phase C work.
- Phase A (fresh-key deploy) and Phase B (anti-cheat docs) are inherited from example_3.
- The convergence root cause (libp2p request_response misuse) is already fixed.
- Phase C (live-join catch-up) is the primary work remaining.
