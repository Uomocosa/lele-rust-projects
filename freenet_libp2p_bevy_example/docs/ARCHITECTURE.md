# ARCHITECTURE

The deterministic-lockstep layout: one shared physics engine run by every client, inputs flowing
between peers, and the freenet contract acting as the enforceable membership + signed audit
ledger.

[[README]] | [[DIFFERENTIATION]] | [[CONTRACT]] | [[NETCODE]] | [[DETERMINISM]] | [[ANTI_CHEAT]] | [[ROADMAP]]

---

## Components

| Component | Where it lives | Responsibility |
|-----------|----------------|----------------|
| **Sim engine** | Every **client** (one hashed, deterministic build) | The ONLY place positions are computed. `advance(state, inputs) -> state`. Pin the build by hash. |
| **Input exchange** | **libp2p** (real-time) | Each peer sends its input for tick *N*; the group gathers all inputs for *N*. Low-latency, direct connections. |
| **State hash broadcast** | **libp2p** (real-time) | After computing `state_{tick}`, broadcast its hash so peers can compare / detect divergence. |
| **Contract** | **freenet** (enforced, persistent) | Membership + **signed input log** (audit & catch-up). Does NOT simulate. See [[CONTRACT]]. |
| **Render layer** | **Client** | Renders engine state with interpolation; captures inputs. Purely client-side I/O. |

## Data flow (per tick)

```
        input capture (client)
                │
                ▼
        libp2p: broadcast own_input[tick]
                │
                ▼
   gather all peers' inputs for tick N  (commit-then-reveal + fixed command delay, see NETCODE)
                │
                ▼
   sim engine: state_{N+1} = advance(state_N, ordered_inputs[N])
                │
                ▼
      libp2p: broadcast state hash of N+1
                │
                ▼
   render engine state (client)  ·  audit hashes (recompute)  ·  commit inputs to contract log
```

- **Nobody sends a position.** The only live-game data on the wire is inputs + state hashes.
- **Ordering** of `ordered_inputs[N]` is the deterministic Option A convention from
  [[NETCODE]] (e.g. sorted by identity) so every peer's engine applies them identically.

## Role boundaries (responsibility split)

| Concern | Owned by |
|---------|----------|
| "What's an allowed membership transition / who may write" | **Contract** (self-certifying) |
| "What inputs are well-formed / monotone / signed" | **Contract** (input log) |
| "What the physics actually does / what a box may become" | **Sim engine** (deterministic, hashed) |
| "Is a peer's committed history honest" | **Peers** recompute + audit against the engine |
| "How fast does it feel" | **libp2p** + client interpolation |

## Transport (hybrid)

- **Real-time path:** libp2p carries inputs and state hashes for live play (low latency).
- **Ledger path:** the freenet **contract** stores the signed input log (per-player, monotone
  `seq`), used for audit and for a rejoining peer to catch up from a committed history without
  reconnecting to every live peer.

## Connectivity of the pieces

```
  Client A  ── libp2p (inputs, state hashes) ──  Client B        (real-time)
     │                                              │
     │          freenet node (membership +           │
     └────────── signed input log contract) ─────────┘           (audit / catch-up)
```

Everything stays symmetric; there is no single server authority. The determinism of the shared
engine + the Option A ordering convention + the uniform command buffer are what keep the peers in
lockstep ([[DETERMINISM]], [[NETCODE]]).

---

## Open / to decide

- Session lifecycle (who can start / join a running session) and tick pacing are refined in
  [[NETCODE]] and [[ROADMAP]] M1/M3.
- Exact input-log schema and bounds in [[CONTRACT]].