# freenet_libp2p_bevy_example_1 — Hybrid Box Game

A two-player box game using the full hybrid networking stack.

## Goal

Prove the complete freenet + libp2p + Bevy integration:

- **Freenet** handles: identity, lobby/discovery, persistent state
- **libp2p** handles: real-time position sync, input events

## Architecture

```
Bevy App
  ├── freenet node ─── DHT ──► lobby, contracts
  └── libp2p swarm ─── direct TCP/QUIC ──► position sync, input
```

## Current Status

M0 (scaffolding), M0.5 (coexistence spike), M1 (local avian2d physics game),
M2 (roster contract on Freenet), and **M3 (libp2p real-time sync)** are done.

The full hybrid stack works end-to-end:

- `contract/` — commutative-merge roster (`BTreeMap<PlayerId, PeerEntry>`):
  union keys, last-write-wins per entry on `updated_at`.
- Embedded Freenet node with a real readiness check — the mainnet path waits
  for at least one active connection before contract ops (no proceeding with
  zero peers), and `connect_client_loop` pulls a refresh `Get` every
  `ROSTER_REFRESH_SECS` so a dropped mainnet `UpdateNotification` is recovered
  instead of leaving the roster stale forever.
- `p2p/` — libp2p swarm (QUIC + TCP, `request_response` bincode snapshots at
  30 Hz on `FixedUpdate`), threaded bridge owning the `!Sync` swarm on a tokio
  task, kinematic remote boxes driven by snapshots with interpolation,
  despawn-on-disconnect.
- `--identity-dir`, `--freenet-local`, `--freenet-gateway` CLI flags (identity
  persistence and a hermetic same-machine mode that bypasses the flaky
  public-mainnet discovery path). The embedded node's UDP port is always
  auto-picked fresh (including on each bootstrap retry) — there is no
  `--p2p-port` override.
- `testing/` — hermetic two-node roster/box tests plus
  `local_two_node_production_sync`, which drives two real production-path app
  instances wired directly and converges + syncs movement deterministically.

### Known limitation (upstream, not this project)

Public-mainnet node discovery is timing-dependent in two distinct ways:

1. **A single contract op can stall or vanish.** `Update` fan-out is fire-and-forget
   (freenet-core PR #2038) and can be silently dropped, so a `Get`/`Update` can take
   minutes. This project survives it by retrying roster setup with capped backoff, the
   on-screen freenet status, and a fast pull refresh (5 s) during startup.

   **The pull refresh does less than this once claimed.** A client `Get` is answered from
   the node's own local copy whenever that node holds valid state and is subscribed or has
   local interest — freenet's serve-DURING gate,
   `client_events::should_serve_local_copy` — which is always true for us after
   `setup_contract` subscribes. The refresh therefore never leaves the machine; upstream
   states it directly in freenet-core#4064: *"Subscribers don't pull on demand — they wait
   for explicit UPDATE."* So the refresh recovers a dropped notification only from state
   the node already applied, and **cannot** discover a peer sitting on a disjoint replica.
   Observed directly in `.local-run/20260820T085231Z-mainnet`: two instances re-read a
   2-entry local replica every 15 s for five minutes while a third held 3 entries.
1. **Node bootstrap itself can fail transiently.** The mainnet can refuse every gateway
   dial for minutes at a time (all NAT traversals failing, `wait_ready` timing out).
   `connect_and_run` retries `start_embedded_node` with capped backoff instead of giving
   up, so the game stays playable single-player and the roster joins on a later retry.
2. **Two concurrent first-`Put`s of a brand-new key can seed disjoint replicas.** freenet's
   Put probes for an existing holder first (summary-first PUT, #4642) and
   reconciles deltas when it finds one, but if the probe misses (the other seed is still
   being placed), the full-state fallback seeds a second independent copy. Those copies
   only reconcile via the periodic InterestSync anti-entropy exchange, which runs on a
   5-minute heartbeat (`INTEREST_HEARTBEAT_INTERVAL` in freenet-core), so a split can look
   permanent for minutes. Client-side mitigation: `setup_contract`'s `not_found_grace`
   window re-checks for an existing seed for up to `SETUP_CONTRACT_GRACE_SECS` before
   `Put`ting, and the mainnet e2e probe staggers the second instance's join by 45 s —
   mirroring real users, who never start an app in the same instant.

The mainnet e2e probe (`testing/tests/e2e_three_node_production_sync.rs`) is `#[ignore]`d —
run it explicitly with `cargo test -- --ignored`. The deterministic gate for the
production startup path is the fully-local `local_two_node_production_sync`.

## Scope

M4 (cross-network relay pool) and M5 (polish: status UI, ed25519 identity
bridge) are explicitly **future work**, not part of this ship. See `TODO.md`.

## Reference

- `TODO.md` — milestone plan and open questions.
- `M2_STEP.md`, `M3_STEP.md` — design writeups for the completed milestones.
