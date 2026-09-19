# CLICKER MAINNET PROBLEMS AND PROPOSED SOLUTIONS

> **OUTDATED wording, see `MAINNET_PROBLEMS_10.md`:** "Freenet dials /
> Freenet addresses / discover each other via Freenet" below means libp2p
> dials seeded by Freenet. Freenet = rooms + occupants seed only.

How the clicker game finds other players without `--dial`: each instance boots an
embedded Freenet client-peer node on the real mainnet and publishes its libp2p
addrs into a per-lobby roster contract (`bincode(namespace, lobby)` params).
Peers dial discovered addrs. No addresses are passed between apps.

Reference implementation: `src/discovery/`. Test: `tests/e2e_local/mainnet.rs`
(`freenet:run-local-mainnet`, `CLICKER_CONTRACT_PARAMS` for reproducible keys).

Observed on runs against the public mainnet (3 same-host instances, one NAT):
2 passes (118s, 254s), 4 fails. Everything below is from those logs.

## Problem 1 — Simultaneous cold start partitions the roster (Freenet level)

Three concurrent first `Put`s of a fresh key seed three disjoint singleton
replica groups. Our nodes learn each other's Freenet addresses from the ring
but direct NAT-traversal between same-public-IP ports fails
(`Outbound handshake failed: max connection attempts`, no hairpin), so the
singletons never intersect: no cross-sibling relays, `BROADCAST_NO_TARGETS`,
bridge subscribes route to non-hosts. Anti-entropy (5-min, neighbor-pair-only)
never pairs the right neighbors.

Evidence: run 1 (fresh key) — 12 min, zero foreign entries anywhere.
Run 2 passed only because instance-2 joined ~2 min late and `Get` pulled
mature state.

Proposed solution: stagger spawns by Freenet readiness — spawn N+1 only after
`discovery: roster connected` appears in N's log (already implemented in the
test). Late joiners pull instead of racing. Matches real usage; nobody
triple-cold-starts the same second.

## Problem 2 — Stale roster entries point at dead ports

`PlayerId` keys are stable (own_id 1/2/3) but libp2p ports are ephemeral per
run. A mature roster hands newcomers the previous run's dead addrs;
`dial_known` dials them (fast fail, harmless) while live addrs propagate
slowly. Worse: an app that trusts the roster blindly dials ghosts first.

Proposed solution: burst announces at startup (immediate + every 5s for
15s, then 30s steady — implemented in `src/discovery/run.rs`), so live addrs
outrun stale ones within seconds. Optional follow-up: skip dialing entries
with `updated_at` older than ~300s.

## Problem 3 — Bridge drops the notifications it waits for

`recv_response` skipped (discarded) `UpdateNotification`s while waiting for a
`SubscribeResponse` (up to 30s). Any foreign state arriving mid-bridge was
lost. Same hazard existed in the 10ms announce ack wait.

Proposed solution (implemented): single drain point. `poll()` is the only
reader and merges both `UpdateNotification` and `GetResponse` states;
announce and bridge are fire-and-forget. Confirmations are ignored.

## Problem 4 — Position publish stalls without mouse movement

`publish_cursor` sent only on movement plus one startup burst (lost before any
peer connects). Idle mice = radio silence for minutes, so player resolution
(`resolve_player` needs a gossip `Move`) never ran. Two runs sat fully
connected for ~5 min with zero resolution, then resolved within seconds of
the first mouse move.

Proposed solution (implemented): 5s position heartbeat in
`src/clicker/bevy_systems/publish_cursor.rs`, movement fast-path unchanged
(15 Hz). Heartbeat traffic is libp2p-local; zero Freenet impact.

## Problem 5 — Game data plane is full-mesh-only

`Click` and `SyncReq/SyncAck` travel by direct `Command::Send` to
roster-known peers (`src/clicker/click.rs:20-27`); gossip carries only
positions, and spawn/resolve require direct roster membership. On a star
topology (2↔3 edge missing) leaves never exchange clicks, so exact-agreement
checks fail even with a working mesh center.

Proposed solution: none implemented — by design. The test must achieve full
mesh (resolution gate below). If partial meshes must work later: spawn cursor
entities opportunistically for gossip senders not in roster (with
`LOBBY_CAP` respected + last-seen expiry to avoid ghosts), or route clicks
over gossip. Both are game-logic changes; deferred.

## Problem 6 — Test gate must assert resolution, not just connection

`connected + tick + ≥1 remote owner` passes on a star mesh minutes before the
game is actually synced (and an earlier revision asserted tick-owner numbers
that can never appear — ticks log advisory hashes, not player numbers).

Proposed solution (implemented): gate on `resolved_count >= 2`
(`cursor resolved peer=` lines) in every instance log, bounded by the
300s timeout. This is true player attribution, the precondition for driving.

## Why freenet_example goes 5/5

Single-channel sync: its data plane IS the Freenet contract (counter ticks →
`Update` → max-merge). No libp2p, no dials, no mesh, no peer→player mapping.
Its gate is "connected + ticking" and its criterion is statistical
accumulation (~1/s over 30s) — three fully-split singletons still pass,
because local ticking never needs convergence. Our exact-agreement bar
(`p1=15 p2=15 p3=16 global=46`, lag spread ≤2s) is strictly stronger and
fails on every transient partition theirs cannot see.

To reach their reliability without weakening the bar: keep stagger + heartbeat
+ resolution gate (done — 118s green run), add the missing bridge re-put leg
(theirs does subscribe AND re-put; ours only subscribes), and accept that
simultaneous cold starts on hairpin-less NAT are Freenet-hard, not app bugs.
