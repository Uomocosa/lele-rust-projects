# Plan: Diagnose & Fix Intermittent Convergence Failure

## Status

**Diagnostic-first.** The plan has been rewritten around a single question, decided *before* any
redesign: **"Is the failure Freenet's architecture, or our code/harness?"** We will not add any
delivery layer (RosterSync, gossipsub, etc.) or make any behavioral change until the evidence
points at a specific mechanism. Adding a protocol to compensate for a bug we haven't proven exists
would be both wasteful and (via a faster delivery path) effectively "cheating past" the real issue.

Two hard rules for everything that follows:
1. **No local-special handling.** A `--bootstrap-roster` / automation-injected discovery path is
   explicitly forbidden — the local run must exercise the identical discovery code path as a
   cross-OS run.
2. **No new delivery layer until proven necessary.** Do not add libp2p roster delivery (Request/
   gossip) as the assumed fix. First prove where the fault is (Phase 1).

## Constraints (non-negotiable)

1. **No special-casing the local run.** Anything that only works because the automation spawns a
   known set of co-located instances is cheating. A fix must work identically when instances run on
   different machines/networks.
2. **Discovery is fully Freenet-driven.** Peers are discovered by subscribing to the same roster
   contract (`ContractKey` = wasm + params). A peer running a different wasm gets a different key
   and is excluded — this is the contract-identity enforcement lever. Never feed libp2p addresses
   in from outside Freenet.
3. **libp2p is the peer-communication layer** (game netcode). It is a candidate *delivery* channel
   only to be evaluated after Phase 1 evidence — never the origin of any address.
4. **Live-join semantics** are kept. Peers may join mid-game; the game must not reset to tick 0
   on a late join.
5. **Success criterion = full state-hash convergence** of all engines, not just mutual peer
   visibility.

## Current Architecture

```
Instance A                                              Instance B
    |                                                       |
    |--[1] Start libp2p swarm (TCP/QUIC) ----------------->|
    |--[2] Start embedded Freenet node -------------------->|
    |--[3] Deploy/subscribe to roster contract ------------>|
    |                                                       |
    |   [Freenet subscription mesh - roster discovery]       |
    |<--- UpdateNotification (roster entries) ------------->|
    |----> Update (own PeerEntry with libp2p addrs) ------->|
    |                                                       |
    |--[4] Bevy: dial_roster_peers reads roster --------->|
    |<---[5] libp2p direct connection established --------->|
    |                                                       |
    |   [libp2p request-response - game netcode]            |
    |<--- NetcodeMsg::Commit / Reveal / StateHash -------->|
    |===> Identical authoritative engine state =============|
```

Two layers:
- **Freenet**: roster/peer discovery only (contract stores `BTreeMap<PlayerId, PeerEntry>`).
- **libp2p**: direct game netcode streams (commit-reveal lockstep protocol).

### Node mode (important — this is already uniform)
`start_embedded_node.rs` runs the MAINNET case as `is_gateway: false`, `skip_load_from_network:
false`, with no public address — each instance joins the real public mainnet as a **client node via
the gateway index**. The embedded nodes do **not** form P2P with each other over loopback. Local and
cross-OS runs take the identical code path. Therefore local runs ARE a faithful test of discovery;
do nothing that treats them differently.

## What "eventual consistency" means in wall-clock (grounded in freenet 0.2.128)

Freenet converges through three mechanisms with very different speeds. This is why a 10-minute
non-convergence is NOT consistent with "just slow anti-entropy" — the 5-minute heartbeat would have
fired.

| Mechanism | Source | Timescale |
|---|---|---|
| Fast push (broadcast) | node applies update → `broadcast_state_change` to connected interested neighbors (executor_impl.rs:644, 2423). **Best-effort by design** — "a missed broadcast heals via the next UPDATE" (executor_impl.rs:2025-2044). | **seconds** — *if* the mesh is connected |
| Anti-entropy reconciliation | `INTEREST_HEARTBEAT_INTERVAL = 300s` (ring/interest.rs:72) — the ~5-min InterestSync heartbeat. State fetching also piggybacks on this cadence. | **~5 minutes** |
| Transport/ring formation | embedded nodes join mainnet via gateways/NAT; flaky (`connect_and_run.rs:6-12`, `RING_TRANSPORT_DESYNC`). | **minutes, or never** |

**Conclusion:** if 3 peers never converge in 10 min, the likely fault is the third tier (nodes
effectively isolated from each other on mainnet) **or a code/harness bug** (contract-identity
mismatch, dead-ended subscription) — NOT the anti-entropy philosophy. Both must be ruled out with
evidence before we touch the architecture.

## Root Cause Hypotheses (UNCONFIRMED — to be disproven by Phase 1)

Observed failure: instance-2 stuck at 1/2 peers (roster had instance-0, never instance-1), leading
to permanent state divergence. Candidate causes, ranked by which we must rule out first:

| # | Hypothesis | Mechanism | Domain |
|---|------------|-----------|--------|
| H1 | **Contract identity mismatch** | Two instances derive different `ContractKey` (different wasm bytes or params) → different logical contracts → can never see each other regardless of Freenet health. | Our code/config |
| H2 | **Subscription dead-ended** | `Get{subscribe:true}` never joined the interest mesh (freenet-core#4414). Node thinks subscribed; neighbors never relay updates; no `UpdateNotification` ever arrives. | Our code (usage) |
| H3 | **Isolated transports on mainnet** | Embedded nodes failed to form ring connections to each other's region (`RING_TRANSPORT_DESYNC`, NAT/gateway refusal). They are not talking at all. | Environment/mainnet |
| H4 | **Genuinely >10 min** | Even connected, delivery missed the window. | Rare; needs measurement |

H1 and H2 are **our fault** and would not be fixed by any new delivery layer — they must be ruled
out first.

## Plan

Uniform, evidence-gated, no behavior change until Phase 1 says otherwise.

### Phase 1: Diagnostic + Control Experiment — Prove Where the Fault Is

**Goal:** Determine, with measurements, whether the non-convergence is Freenet's delivery /
environment or a code/harness bug. **No behavior change to the roster/protocol logic** — add
logging and a control topology only.

**Step 1 — Prove the contract is identical (rules out H1).**
- Log the full `ContractKey` + params digest per instance at setup. `setup_contract.rs` already has
  `contract_key` in scope — emit it (target `roster::change`, plus params digest).
- Assert/log that all N instances share the same key. If two differ → that is the whole bug; fix
  it (trivially) and stop.

**Step 2 — Measure real cross-node delivery latency (rules out H4 / confirms H3).**
- Timestamp `T0` when each instance publishes its own entry (heartbeat/`Update` in
  `connect_client_loop.rs`).
- Timestamp when each *other* instance's node **applies** that entry: log on `UpdateNotification`
  receipt (`connect_client_loop.rs:118`) with the state digest; also log on `GetResponse` the
  digest actually seen.
- Record the wall-clock delta per (publisher → observer) pair.
- Interpretation:
  - deltas in **seconds** → Freenet delivers fine when connected → bug is elsewhere (H1/H2).
  - deltas = **never / minutes** → the observers never saw the publisher's replica (H3/transport)
    or were never subscribed (H2).

**Step 3 — Watch node/ring health over time (rules out H2/H3).**
- Reuse the `NodeDiagnostics` API already used in `connect_client_loop.rs::log_node_diagnostics`
  and poll it periodically for the whole run, logging: `active_connections`, ring connection
  count, and interest/subscription state for the roster contract.
- This tells us whether the nodes are genuinely connected to the mesh and whether the subscription
  actually armed — i.e., *why* a broadcast did or did not land.
- Also log `RING_TRANSPORT_DESYNC`-class events with the ring/transport counts at the moment they
  occur.

**Step 4 — Connected control run (decisive experiment).**
- Run the **same binary** on a **guaranteed-connected** topology — all clients dial a single
  in-process gateway via the `--freenet-gateway` hermetic path in `start_embedded_node.rs`
  (or an equivalent where transport is known-good), using distinct identity dirs and a distinct
  `--contract-params` namespace so it is isolated from production.
- If it converges in **seconds** → contract + app logic proven correct; the culprit is mainnet
  transport isolation (H3). *Only then* does a potential delivery layer have a real job.
- If it **also fails** hermetic → it is an app/subscription/identity bug (H1/H2) → fix the code; no
  delivery layer is needed.

**Files to modify (logging only, unless a bug is found):**
- `src/roster/connect_client_loop.rs` — log every roster entry change; log `T0` publish + receive
  timestamps; log contract key/params digest.
- `src/roster/setup_contract.rs` — log full `ContractKey`.
- `src/roster/bevy_systems/poll_freenet_events.rs` — log merge details.
- `src/roster/connect_and_run.rs` / `start_embedded_node.rs` — periodic `NodeDiagnostics` sampling,
  ring/transport counts.
- `src/p2p/run.rs` — log every dial attempt, connection success/failure/disconnect.
- `src/boxes/bevy_systems/netcode_tick.rs`, `src/netcode/lockstep_advance_to.rs` — log lockstep
  state / missing reveals.

**New log targets:** `roster::change`, `p2p::connect`, `freenet::ring`, `freenet::interest`,
`lockstep::state`.

**Deliverable:** a written verdict (H1/H2/H3/H4) backed by the steps above, which determines which
of the following phases (if any) apply. **Do not proceed past Phase 1 without this verdict.**

**Risk:** None (logging-only) except where a bug is found, which is the point.

---

### Phase 2 (GATED — only if Phase 1 shows H3 or H4): libp2p gossipsub member topic

**Only if** Phase 1 proves the roster logic is sound and the failure is delivery/topology **with
reachable peers** (i.e., a reliable channel would resolve it). Skip entirely if Phase 1 finds H1/H2.

**Rationale / theory:** discovery is the "chicken-and-egg of addresses"; once a peer knows even ONE
member via Freenet it can join a fast broadcast overlay and learn the rest reliably. Freenet stays
the source of truth + identity gate; gossipsub is a fast, reliable *delivery* of entries that peers
already published to the contract. This is a slow-authoritative-bootstrap + fast-gossip-overlay
pattern, uniform across local and cross-OS.

**Design:**
- Add `"gossipsub"` to the libp2p features in `Cargo.toml` (not currently enabled).
- Add a `gossipsub::Behaviour` to `behaviour.rs` / `behaviour_new.rs`.
- Topic name derived from `Params.namespace` (isolation: different game = different topic).
- On startup each instance **subscribes** to the topic and **publishes its own `PeerEntry`**
  (player_id + peer_id + addrs).
- `run.rs` handles `gossipsub::Event::Message` → decodes `PeerEntry` → routes into the roster merge.
- Roster merge single point: **one shared `roster::merge_into(&mut Roster, entries)`** helper. Both
  the Freenet path (`poll_freenet_events`) and the gossipsub path call it — never two parallel
  writes into `Roster`.
- Seeding: the first Freenet-learned edge is used to join the gossip mesh (Freenet must deliver at
  least one edge per connected component; gossipsub floods the rest in seconds).
- Game netcode (`request_response`) **unchanged** — gossipsub carries only the control-plane member
  announcements, not per-tick data.

**Trust model:** gossip is untrusted transport; trust comes from `PeerEntry` signatures and the
authoritative Freenet contract (the whitelist of authorized keys). For the cooperative test a
lenient merge is acceptable (eventual contract prunes phantoms); hardening = verify each gossip
entry against the contract before trusting.

**Risk:** Medium. Libp2p handles forwarding/mesh repair; app code is small (publish + merge).

---

### Phase 3 (DEFERRED): Live-Join State Catch-Up via Re-Baseline Snapshot

Required only to meet full state-hash convergence under live-join semantics. Do not design/build
until Phase 1 is settled and the convergence gate is green for the already-together case.

**New messages in `src/p2p/netcode_msg.rs`:**
```rust
RequestSnapshot { }
Snapshot {
    tick: u64,
    bodies: BTreeMap<engine::PlayerId, (f32, f32, f32, f32)>,  // x, y, vx, vy — FULL EngineSimState
    participants: Vec<engine::PlayerId>,
    from: engine::PlayerId,
}
```
`restore()` needs velocities (`EngineSimState`, engine_sim_state.rs) to reproduce the exact
trajectory; the positions-only `Snapshot` (snapshot.rs) is NOT sufficient.

**Handshake:** on first connect, late joiner C sends `RequestSnapshot`; each established peer
replies with its authoritative `Snapshot`; C picks the deterministic authority (lowest
`PlayerId.from`) and cross-checks its state hash against a second peer before adopting.

**Re-baseline:** existing peers never saw C's inputs, so C's body is injected deterministically at
snapshot tick T: all peers spawn C's body at default position at T, C `restore()`s the authoritative
state + seeds its own body; all set `applied_through = T`, participants = snapshot set + C; continue
forward. Hashes reconverge from T onward; game does not reset to 0.

**Risk:** High. Verify with a dedicated integration test (two converge, third joins and converges).

---

### Phase 4 (OPTIONAL — only if Freenet stays on the critical path): Ring Health Monitoring + Retry
- Log ring/transport/interest counts; early-abort clearly-failed node startups.
- `connect_and_run.rs` already retries node startup with capped backoff (keep). Bounded
  automation-level retry on convergence timeout. Uniform.
- **Low priority** if Phase 2 (gossipsub) makes discovery deterministic; likely unnecessary.

---

## Implementation Order (evidence-gated)

| Order | Phase | Gate | Time | Risk |
|-------|-------|------|------|------|
| 1 | Phase 1 — Diagnostic + Control Experiment | — | 1-2 days | None (logging) |
| 2 | **Decision** — write the H1/H2/H3/H4 verdict | Phase 1 results | — | — |
| 3 | H1/H2 fix (code/config), **OR** | verdict = H1/H2 | small | Low |
| 4 | Phase 2 gossipsub, **OR** | verdict = H3/H4 | ~1-2 days | Medium |
| 5 | Phase 3 live-join catch-up | convergence gate green | 2-3 days | High |

**The critical decision gate is after Phase 1.** Do not proceed to Phase 2 or 3 until the verdict is
written and reviewed.

## Expected Outcome

- **Phase 1:** an evidence-backed answer to "is it Freenet, or our harness?", with per-pair
  delivery latencies and per-instance contract key + node/ring health. No speculative redesign.
- **If H1/H2:** a small, honest code fix — Freenet's mesh was fine, we were misusing it.
- **If H3/H4:** gossipsub member topic makes discovery deterministic in seconds while Freenet
  remains the identity/authorization gate; possibly ring-health + retry hardening.
- **Phase 3 (deferred):** full state-hash convergence under live join.

## Key Files Reference

| File | Purpose |
|------|---------|
| `src/roster/connect_client_loop.rs` | Roster loop: heartbeat, refresh, absorb; Phase 1 timestamps |
| `src/roster/setup_contract.rs` | Deploy/subscribe; Phase 1 contract-key/digest logging |
| `src/roster/start_embedded_node.rs` | Start in-process node; hermetic control topology flag |
| `src/roster/connect_and_run.rs` | Node startup retry/backoff; diagnostics sampling hook |
| `src/roster/bevy_systems/poll_freenet_events.rs` | Drain roster events → merge (one writer) |
| `src/roster/merge_roster.rs`, `prune_stale.rs` | Deterministic merge + TTL (used by `merge_into`) |
| `src/roster/roster.rs` | Bevy `Roster` Resource; shared `merge_into` target |
| `src/p2p/behaviour.rs`, `behaviour_new.rs` | NetworkBehaviour — add gossipsub if Phase 2 |
| `src/p2p/run.rs` | Swarm event loop; handle gossipsub Message (Phase 2) |
| `src/p2p/netcode_msg.rs` | NetcodeMsg enum (add RequestSnapshot/Snapshot for Phase 3) |
| `src/p2p/bevy_systems/dial_roster_peers.rs` | Dial peers from roster |
| `src/boxes/bevy_systems/netcode_tick.rs` | Per-tick lockstep pipeline |
| `src/netcode/lockstep.rs` | Lockstep state (participants, applied_through) |
| `src/engine/engine_sim_state.rs` | Full restorable state (what Snapshot must carry) |
| `mainnet_automation_3/src/launch_instances.rs` | Launch instances; control-topology wiring |
| `mainnet_automation_3/src/wait_all_converged.rs` | Convergence check (420s timeout) |
| `mainnet_automation_3/src/applied_player_ids.rs` | Current gate (peer-visibility) — see note below |

## Note on the Convergence Gate
`wait_all_converged.rs` + `applied_player_ids.rs` gate on mutual peer visibility (seen N-1 distinct
`received peer input`). Phase 3 introduces the real state-hash mechanism; extend the gate to assert
equal `StateHash` logs once Phase 3 lands. **Do not weaken the gate to make tests pass.**
