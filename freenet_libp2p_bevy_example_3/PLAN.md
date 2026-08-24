# Plan: Fix Intermittent Convergence Failure

## Status
Refined after review. Doomed assumptions removed. A `--bootstrap-roster` / local-special
discovery path is **explicitly forbidden** — the local run must exercise the identical
discovery code path as a cross-OS run. Freenet is the discovery layer and the identity gate.

## Constraints (non-negotiable)

1. **No special-casing the local run.** Anything that only works because the automation
   spawns a known set of co-located instances is cheating. A fix must work identically when
   instances run on different machines/networks.
2. **Discovery is fully Freenet-driven.** Peers are discovered by subscribing to the same
   roster contract (`ContractKey` = wasm + params). A peer running a different wasm gets a
   different key and is excluded — this is the contract-identity enforcement lever. Never
   feed libp2p addresses in from outside Freenet.
3. **libp2p is the peer-communication layer** (game netcode + roster anti-entropy). It may
   relay roster entries that peers *already published* to the Freenet contract, to cover a
   missed `UpdateNotification`. It must not be the origin of any address.
4. **Live-join semantics** are kept. Peers may join mid-game; the game must not reset to
   tick 0 on a late join.
5. **Success criterion = full state-hash convergence** of all engines, not just mutual
   peer visibility.

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
`start_embedded_node.rs` runs the MAINNET case as `is_gateway: false`,
`skip_load_from_network: false`, with no public address — each instance joins the real
public mainnet as a **client node via the gateway index**. The embedded nodes do **not**
form P2P with each other over loopback. Local and cross-OS runs take the identical code
path. Therefore local runs ARE a faithful test of discovery; do nothing that treats them
differently.

## Root Cause Analysis

### Failure: Instance-2 stuck at 1/2 peers

Three cascading problems:

#### 1. Embedded Node Startup Failure (90s delay)
Instance-2's freenet node hit `RING_TRANSPORT_DESYNC: transport_connections=2,
ring_connections=0`. Transport connections existed but were never promoted to ring topology.
After the 90s `wait_ready` timeout the node aborted and retried (`connect_and_run.rs`).
Mainnet gateway bootstrap is flaky by nature (documented in `connect_and_run.rs:6-12`).
The successful run had zero embedded node failures.

#### 2. Incomplete Roster After Late Join (CORE BUG)
After retry, instance-2's roster only had instance-0, never instance-1. The roster flow has
a critical gap documented in `connect_client_loop.rs:227-235`:

> "Subscribers don't pull on demand — they wait for explicit UPDATE." ... Healing a real
> split depends on an inbound broadcast or the ~5-minute InterestSync anti-entropy heartbeat,
> neither of which this loop can force.

| Mechanism | What It Does | Why It Fails |
|-----------|-------------|-------------|
| Heartbeat | Republishes LOCAL entry every 60s | Only publishes own entry, not full roster |
| Get refresh | Re-reads contract state | Only reads local node state (freenet-core#4064), not network state |
| UpdateNotification | Push from Freenet | If missed during the 90s delay, no on-demand recovery exists |

The `absorb` function correctly merges incoming views, but can only merge what it *receives*.
If the initial `UpdateNotification` containing instance-1's entry was missed, the roster
stays incomplete — and Freenet has no on-demand pull to fix it.

#### 3. Permanent State Desync (live-join + live-join divergence)
With incomplete roster:
- Instance-2 broadcasts commits/reveals to instance-0 only.
- Never receives reveals from instance-1 (not in roster).
- Late joiner starts at tick 0 with default positions while the engine is at tick N; there
  is no state catch-up mechanism.

## Plan

Uniform mechanisms only. No local/cross-OS branching in behavior — only knobs that apply to
both deployments.

### Phase 1: Diagnostic Logging
**Goal:** See exactly what's happening at each stage. No behavior change.

**Files to modify:**
- `src/roster/connect_client_loop.rs` — Log every roster entry change (add/remove/update) with full entry details.
- `src/p2p/run.rs` — Log every dial attempt, connection success/failure/disconnect with peer IDs.
- `src/boxes/bevy_systems/netcode_tick.rs` — Log lockstep state (committed/revealed/missing peers) per tick.
- `src/roster/bevy_systems/poll_freenet_events.rs` — Log roster merge details.
- `src/netcode/lockstep_advance_to.rs` — Log when default actions are used for missing reveals.

**New log targets:**
- `roster::change` — entry-level roster mutations.
- `p2p::connect` — connection lifecycle.
- `lockstep::state` — per-tick protocol state.

**Risk:** None.

---

### Phase 2: RosterSync Anti-Entropy over libp2p
**Goal:** Make roster convergence deterministic in seconds, once at least one peer-to-peer
edge exists. Freenet remains the origin of every entry; libp2p merely delivers entries that
peers already published to the contract, covering missed `UpdateNotification`s.

**In `src/p2p/netcode_msg.rs`** add:
```rust
RosterSync { entries: Vec<(engine::PlayerId, roster::PeerEntry)> }
```
Note: `NetcodeMsg` moves between `p2p` and `roster` domain types — place the variant and its
serde derivation carefully (both domains are in-crate; keep imports to `engine` only in this
file or extract as needed).

**In `src/p2p/run.rs`** — no change to message plumbing; `RosterSync` rides the existing
`request_response::Behaviour<NetcodeCodec>` just like `Commit`/`Reveal`.

**In `src/p2p/bevy_systems/dial_roster_peers.rs`**:
- On a new `PeerConnected` event, send full roster as `NetcodeMsg::RosterSync`.
- Track a periodic 5s deadline that broadcasts the full roster to all connected peers.

**In `src/boxes/bevy_systems/netcode_tick.rs`**:
- Handle inbound `RosterSync`: `absorb`-merge into `roster::Roster` (reuse
  `merge_roster.rs` + `prune_stale`), then write it back through the same path the Freenet
  events use so `Roster` (a Bevy `Resource`) stays consistent and `dial_roster_peers` picks
  up any new addresses.
- Centralize roster mutation (Freenet `UpdateNotification`, `GetResponse`, and libp2p
  `RosterSync`) through one merge entry point so all three sources join the same
  `RosterState`.

**Why this works:** libp2p connections are direct and reliable; relaying already-published
Freenet entries over them heals delivery gaps. This is NOT a bypass of Freenet discovery —
every address still originates from the contract.

**Risk:** Medium. New message variant; follow the existing `NetcodeCodec` pattern.

---

### Phase 3: Live-Join State Catch-Up via Re-Baseline Snapshot
**Goal:** A late joiner reaches state-hash convergence with existing peers WITHOUT resetting
the game to tick 0. This is required by the constraints (live-join + full state-hash
convergence).

This is dynamic-membership lockstep, the hardest part. Design carefully; build it only after
Phases 1, 2, 4, 5 are green for the already-together case.

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

**Why full state (including velocity):** `restore()` re-steps the deterministic sim; it
needs velocities (`EngineSimState`, engine_sim_state.rs) to reproduce the exact trajectory.
The existing `Snapshot` (snapshot.rs) is positions-only and is NOT sufficient — do not use it
for transfer.

**Handshake:**
1. On first connect to an established peer (via `PeerConnected`), the late joiner C sends
   `RequestSnapshot` to every connected peer.
2. Each established peer replies with its current authoritative `Snapshot` (its
   `EngineSimState` + participant set + its own id).
3. C selects the deterministic authority (lowest `PlayerId.from` among the responses) and
   **cross-checks** its state hash against a second peer's response before adopting, to avoid
   adopting a diverged peer. If the authority and a second peer disagree, C takes no snapshot
   and reports/logs the divergence (Phase 1 logging aids this).

**Re-baseline (the join moment):**
Existing peers never saw C's input history, so they cannot replay it. Therefore C's body is
injected deterministically at the snapshot tick T:
1. All peers (A, B, C) **deterministically spawn C's body** at its default position at tick T
   (see `engine_spawn_player.rs`). This is an explicit, agreed event — not part of the
   pre-T history.
2. C `restore()`s the authoritative state (A/B bodies) and seeds its own freshly-spawned body
   at the same default position/tick.
3. Every peer sets:
   - engine clock and lockstep `applied_through = T`
   - lockstep `participants` = snapshot.participants + C (lockstep.rs:14)
4. All peers continue from T forward as equal participants, so state hashes reconverge from T
   onward.

**Clock/state alignment on C:** `Lockstep::applied_through = T`; engine `EngineSimState{tick:
T} = restored`.

**Risk:** High. Determinism and membership edge cases; verify with a dedicated integration
test (two peers converge, then a third joins and converges) before wiring into the automation.

---

### Phase 4: Node / Ring Health Monitoring
**Goal:** Detect and recover from node-startup issues early and uniformly.

**Changes:**
- Log ring connection count, transport connection count, interest sync status on startup and
  on `wait_ready` failure.
- `connect_and_run.rs` already retries node startup with capped backoff (keep it). Add
  visibility so clearly failed runs surface fast instead of stalling the full timeout.

**New log targets:**
- `freenet::ring` — ring connection count, transport connection count.
- `freenet::interest` — interest sync status, peer interests.

**Risk:** Low. Monitoring only.

---

### Phase 5: Automation-Level Retry
**Goal:** Bound the remaining mainnet flakiness with a uniform retry (also applies to cross-OS).

**In `mainnet_automation_3/src/launch_instances.rs` (or a new runner module):**
- On convergence timeout, kill all instances.
- Wait 5s for ports to free.
- Retry up to 2 times, logging each attempt.

**Why needed:** mainnet ring/node startup is non-deterministic even with all fixes
(`connect_and_run.rs:6-12`). A bounded, logged retry is a reliable finishing layer.

**Risk:** Low. Automation-level only.

---

## Implementation Order

| Order | Phase | Time | Risk | Impact |
|-------|-------|------|------|--------|
| 1 | Phase 1 — Diagnostics | 1-2h | None | Visibility |
| 2 | Phase 2 — RosterSync | 3-4h | Medium | Core convergence fix |
| 3 | Phase 4 — Ring Health | 1h | Low | Early abort |
| 4 | Phase 5 — Retry | 1h | Low | Reliability |
| 5 | Phase 3 — Live-Join Catch-Up | 2-3 days | High | Full state-hash convergence, live join |

## Expected Outcome

After Phases 1, 2, 4, 5:
- Full visibility of each stage.
- All peers discover each other deterministically via libp2p `RosterSync` relay of
  Freenet-published entries.
- Clearly failed runs abort early; remaining flakiness handled by bounded retry.
- The already-together case converges to mutual peer visibility.

After Phase 3:
- A late joiner reaches **full state-hash convergence** with existing peers via the re-baseline
  snapshot, preserving live-join (no reset to tick 0).

Phase 2 makes mutual-visibility convergence deterministic. Phase 3 makes full state-hash
convergence deterministic under live join. Phase 5 is a safety net for unavoidable mainnet
flakiness.

## Key Files Reference

| File | Purpose |
|------|---------|
| `src/roster/connect_client_loop.rs` | Roster loop: heartbeat, refresh, absorb |
| `src/roster/setup_contract.rs` | Deploy/subscribe to roster contract |
| `src/roster/start_embedded_node.rs` | Start in-process Freenet node (uniform client-node mode) |
| `src/roster/connect_and_run.rs` | Node startup retry/backoff |
| `src/roster/bevy_systems/poll_freenet_events.rs` | Bevy system: drain roster events |
| `src/roster/merge_roster.rs` | Deterministic roster merge (absorb uses this) |
| `src/roster/prune_stale.rs` | TTL-based roster pruning |
| `src/roster/roster.rs` | Bevy `Roster` Resource |
| `src/p2p/run.rs` | libp2p swarm event loop |
| `src/p2p/behaviour.rs` | NetworkBehaviour (`request_response::NetcodeCodec`) |
| `src/p2p/netcode_msg.rs` | NetcodeMsg enum (add RosterSync, RequestSnapshot, Snapshot) |
| `src/p2p/netcode_codec.rs` | bincode length-prefixed codec |
| `src/p2p/bevy_systems/dial_roster_peers.rs` | Dial peers from roster; send RosterSync |
| `src/boxes/bevy_systems/netcode_tick.rs` | Per-tick lockstep pipeline; handle RosterSync/Snapshot |
| `src/netcode/lockstep.rs` | Lockstep state (participants, applied_through) |
| `src/netcode/lockstep_advance_to.rs` | Apply ticks with missing-reveal handling |
| `src/engine/engine_sim_state.rs` | Full restorable state (tick + bodies w/ velocity) — what Snapshot must carry |
| `src/engine/engine_spawn_player.rs` | Deterministic player spawn (used for re-baseline injection) |
| `src/engine/restore` | Full state restoration |
| `mainnet_automation_3/src/launch_instances.rs` | Launch all instances |
| `mainnet_automation_3/src/wait_all_converged.rs` | Convergence check (420s timeout) |
| `mainnet_automation_3/src/applied_player_ids.rs` | Current gate (peer-visibility) — see note below |

## Note on the Convergence Gate
`wait_all_converged.rs` + `applied_player_ids.rs` currently gate on mutual peer visibility
(seen `received peer input ... player_id=` for n-1 peers). That gate is a necessary but NOT
sufficient proxy for full state-hash convergence. Phase 3 introduces the mechanism for real
state-hash convergence; consider extending the automation to also assert equal `StateHash`
logs between instances once Phase 3 lands, so the gate reflects the true success criterion.
Do not weaken the gate to make tests pass.
