# freenet_boxes — TODO

Shared box arena on the hybrid stack. Freenet = identity, lobby, persistent
state. libp2p = real-time position sync.

## Architecture

    Bevy App (main thread)          dedicated thread
      │                               │
      ├── freenet embedded node ──────┼── DHT ──► roster contract
      │     (tokio runtime)           │           (who is here, how to dial them)
      └──── mpsc channels ────────────┴── libp2p swarm
                                            ├─ QUIC direct ──► snapshots @ 30 Hz
                                            └─ circuit relay (fallback, ~30% of pairs)

Freenet carries the *slow, durable* facts: the player roster, each peer's
`PeerId` and candidate multiaddrs. libp2p carries the *fast, disposable*
facts: box transforms. Nothing real-time ever touches a contract.

**Target: 2+ machines on different networks / behind different NATs.** Not a
LAN demo. That drives the whole libp2p feature set.

## Constraints (non-negotiable)

- The contract's `update_state` MUST be a commutative monoid: per-key union of
  the roster, per-entry last-write-wins on a timestamp. Do not copy
  `clicker_contract`'s replace-the-whole-state logic.
- Each peer is authoritative over its own box ONLY. Local box is
  `RigidBody::Dynamic`; remote boxes are `RigidBody::Kinematic` (or
  transform-only) and are driven by incoming snapshots. No lockstep, no
  deterministic simulation.
- Snapshot rate 30 Hz on `FixedUpdate`, decoupled from frame rate.
- **Nothing on the Bevy schedule may block.** No `blocking_recv`, no
  `lock().unwrap()` round trips into the swarm thread — only `try_recv()`.
  (`libp2p_bevy_example`'s `get_connected_peers` gets this wrong; don't copy.)
- The swarm loop must be driven by a real executor (`with_tokio()` /
  `SwarmConfig::with_executor`), continuously — not polled on a heartbeat.
- **Relayed connections are a supported steady state**, not an error. ~30% of
  peer pairs never hole-punch. Show it in the UI, keep playing.
- No `.unwrap()` / `.expect()` / `panic!()`; `thiserror` domain enums.
- Every non-trivial file gets a `test_usage` test (`lele-syntax-rs`).

## Milestones

### M0 — Project scaffolding ✅ DONE 2026-08-10
- [x] `git mv freenet_libp2p_bevy_example freenet_libp2p_bevy_example_1`
- [x] Delete old `src/`; keep `OBJECTIVE.md`, `unruled_examples/`
- [x] Copy `.cargo/config.toml` (target-dir redirect — spaces in path break
      jemalloc), `Makefile.toml` from `freenet_bevy_example_2`.
      **`build.rs` deferred to M2** — it exists in example_2 only to build
      the contract WASM, and this project has no `contract/` yet.
- [x] Drop the stale `run-publisher` / `run-subscriber` `--role` tasks from
      `Makefile.toml` (also dropped the contract-build tasks — no contract
      exists yet; re-add in M2 alongside `build.rs`)
- [x] `Cargo.toml`: bevy 0.19, avian2d 0.7, freenet 0.2, freenet-stdlib 0.8,
      tokio, tokio-tungstenite 0.27, serde, bincode, thiserror, tracing.
      No `bevy_tweening`, no gossipsub, no kad.
      `libp2p = { version = "0.56", features = ["tokio", "quic", "tcp",
      "noise", "yamux", "dns", "identify", "ping", "autonat", "relay",
      "dcutr", "request-response", "macros"] }`
- [x] Update `OBJECTIVE.md` status
- [x] `cargo build --all-targets` green (placeholder `src/lib.rs`; M1 adds
      real content)

### M0.5 — Coexistence spike ✅ DONE 2026-08-10
Done by fixing `freenet_libp2p_example` in place (it previously declared
`freenet` as a dependency and never used it). Full write-up:
`freenet_libp2p_example/OBJECTIVE.md`.

- [x] Embedded freenet node + libp2p swarm in one process, 60 s
- [x] **Verdict: COEXISTENCE = YES.** 120 pings / 0 errors alongside a freenet
      node that simultaneously joined the public network
- [x] **One `#[tokio::main]` runtime is enough.** freenet spawns onto the
      ambient runtime; `SwarmBuilder::with_tokio()` does too. No nested
      `Runtime::new()`, no dedicated thread needed for correctness, no
      executor conflict. Both driven from one `tokio::select!` loop.
- [x] **No UDP conflict.** freenet bound `[::]:60302`, QUIC bound
      `127.0.0.1:49444`. QUIC is safe to use.

> **Still keep the swarm on its own thread in this project** — not because of
> freenet, but because Bevy owns the main thread and must never block. The
> spike only proves the runtimes don't fight.

#### ⚠️ Finding that changes M2: `NotFound` is a FALSE readiness signal
The first Get returned in **1.3 ms**, while the node's first gateway handshake
completed **38 ms later** — the response came back before the node had joined
the network. Every later Get for the same key took **6.4–7.2 s**. A Get against
an empty ring short-circuits to `NotFound` locally; once the ring has peers it
actually searches. (Confirmed by correlating `Registered transaction` →
`Delivering result` on matching transaction ids, independent of client clocks.)

- This is what example_2's `sleep(20s)` is really working around, and
  "we got a response, so we're ready" does **not** replace it.
- **A joining player can read an empty roster and conclude nobody is online.**
  Roster reads must be gated on ring/connection state, or retried with a floor
  — never trusted on the first response during startup.
- [ ] M2 must find the real readiness signal (ring size / connected peer count
      via a node query) instead of a sleep or a first-response check.

#### Other findings
- **Freenet contract ops cost ~7 s, not milliseconds.** Measured repeatedly on
  a live ring. This settles the "is libp2p really needed" question: it is, and
  by a factor of ~200 against the 30 Hz snapshot budget. It also means the M2
  join flow (Get roster → Put own entry) is a multi-second operation — the box
  must spawn and be playable before it completes, not after.
- The embedded `is_gateway: true` node **joins the public Freenet network** and
  relays subscribe traffic for unrelated contracts. It is not isolated.
  Relevant before running many instances in tests.

### M1 — Local game, no networking ✅ DONE 2026-08-10
- [x] `src/boxes/` domain: `plugin.rs` + `plugin_build.rs` delegate pair
- [x] `PhysicsPlugins::default()` (avian2d), gravity, a static ground collider,
      side-view 2D camera
- [x] `player.rs` (`Component { id: PlayerId }`), `local_player.rs` marker
- [x] `spawn_box.rs` — core fn; box = `RigidBody::Dynamic` + `Collider::rectangle`
      + `LockedAxes::ROTATION_LOCKED` + a per-player color
- [x] `move_box.rs` / `jump_box.rs` — core logic fns taking direction/intent
- [x] `bevy_systems/read_input.rs` — WASD **and** arrows for move, Space for
      jump; thin wrapper calling the core fns (Command pattern from the bevy
      skill)
- [x] Grounded check before jump (shapecast or contact query) so you can't fly
      — `SpatialQuery::cast_ray` straight down from the box, excluding itself
- [x] `test_usage` per file; headless `App` tests for move/jump/spawn
- [x] **Checkpoint: one box, keyboard-controlled, jumps, lands. Runs solo.**
      Verified visually — spawned `cargo run --bin bevy_freenet`, screenshotted
      the window: box rendered and resting on the ground under gravity. WASD/
      arrow/Space input logic verified via the `read_input` and `plugin`
      `test_usage` tests (no way to synthesize real key events through the
      desktop-control tooling available in this session).

### M2 — Roster contract on Freenet ✅ DONE 2026-08-10
Full writeup and design rationale: `M2_STEP.md`.

- [x] `contract/src/lib.rs`: state = `BTreeMap<PlayerId, PeerEntry>` where
      `PeerEntry { peer_id: String, addrs: Vec<String>, updated_at: u64 }`
      — `addrs` is plural: a peer publishes its LAN addr, its observed public
      addr (from `identify`), and its `/p2p-circuit` relay addr, and the
      dialer tries all of them
- [x] `update_state` = merge: union keys, on collision keep higher
      `updated_at`. Unit-test commutativity + idempotence + associativity
      explicitly (apply A then B == B then A == both twice)
- [x] `validate_state`, `summarize_state`, `get_state_delta` — delta
      currently returns the full state (mirrors the clicker contract's
      approach and the M2_STEP.md plan); "delta = only the missing
      entries" is a possible follow-up optimization, not required for the
      checkpoint
- [x] `build.rs` + `Makefile.toml` `build-contract`/`copy-wasm` tasks
      restored (deferred in M0 since no `contract/` existed yet). Found and
      fixed a real bug while wiring this up: the nested `cargo build
      --target wasm32-unknown-unknown` inherited `RUSTFLAGS`/
      `CARGO_ENCODED_RUSTFLAGS` from the outer build (mold linker, meant
      only for the host `x86_64-unknown-linux-gnu` target per
      `.cargo/config.toml`), and env-var rustflags apply to every rustc
      invocation regardless of target — broke the wasm link with
      `unknown argument: -fuse-ld=mold`. Fixed via `.env_remove(...)` on
      the child `Command`. **`freenet_bevy_example_2/build.rs` has this
      same latent bug** (reproduced there too) — not fixed, out of scope
      for this project, but worth flagging upstream.
- [x] Reuse `src/freenet/*` from example_2 verbatim (websocket client, error
      enum) — copied as-is, added the missing `futures-util`/`http`/
      `tempfile` deps it needs; no logic changes
- [x] Embedded node startup: replace the hardcoded 20 s sleep with a **real
      readiness check on ring/connection state**. See the M0.5 finding — a
      successful Get is not readiness, and an empty roster read during startup
      is indistinguishable from "nobody is online". Implemented as
      `FreenetClient::wait_ready(min_active_connections, timeout)` in
      `src/freenet/freenet_client_wait_ready.rs`, polling
      `ClientRequest::NodeQueries(NodeQuery::NodeDiagnostics{
      config: NodeDiagnosticsConfig::basic_status() })` until `node_info` is
      populated and `network_info.active_connections >= min_active_connections`
- [x] `roster.rs` resource (`src/roster/roster_resource.rs` — renamed from
      the planned `roster.rs` to avoid clippy's `module_inception` lint
      against `roster::roster::Roster`); `poll_freenet_events` **fully
      drains** the channel each frame via `while let Ok(event) =
      events.receiver.try_recv()` (example_2 returns after one event — did
      not copy that bug)
- [x] `testing/` crate + integration test: two nodes, both join, both see a
      2-entry roster. New work (example_2's `testing::TestNode` never
      actually joined two nodes — each test always started an isolated
      single-node network): `TestNode::start_gateway`/`start_peer`, the
      peer's `--gateway "ip:port,hex-pubkey"` built from the gateway's
      `TransportKeypair::public()` via `hex::encode`. Found and fixed a
      real bug along the way: `NetworkArgs` has *two* port fields —
      `public_port` (advertised) and `network_port` (actually bound
      locally, defaults to 31337) — setting only `public_port` left every
      test node binding the same default `network_port` and NAT traversal
      failed with "max connection attempts reached". Fixed by setting
      `network_port: Some(public_port)` too, in both `testing/` and
      `roster::start_embedded_node`. Test:
      `testing/tests/two_node_roster.rs`
- [x] **Checkpoint: two app instances each see the other in the roster; boxes
      spawn for every roster entry but do not move.** Roster→roster
      verified live in `testing/tests/two_node_roster.rs`. Box-spawning
      added as `roster::bevy_systems::spawn_roster_boxes` (chained after
      `poll_freenet_events`): spawns a non-local `boxes::spawn_box` for
      every roster entry not already represented by a `boxes::Player`
      entity — not yet exercised by a live two-Bevy-app run (no GUI
      harness for that in this session), but covered by a headless `App`
      test

### M3 — libp2p real-time sync (same-machine / LAN first)
- [ ] `src/p2p/swarm.rs`: `SwarmBuilder::with_new_identity().with_tokio()
      .with_tcp(..)?.with_quic().with_dns()?.with_relay_client(noise, yamux)?
      .with_behaviour(..)?.with_swarm_config(..).build()`
- [ ] Behaviour struct: `request_response` (custom bincode `Codec`,
      `ProtocolSupport::Full`, protocol `/freenet-boxes/positions/1.0.0`),
      `identify`, `ping`, `autonat`, `relay::client`, `dcutr`
- [ ] Threading bridge (libp2p skill §8): `Swarm` is `!Sync` — own it on a
      dedicated thread running a tokio runtime, `select_next_some()` in a
      tight loop, push events out over `mpsc::unbounded_channel`. Bevy drains
      with `try_recv()`. **No blocking calls back into the swarm thread.**
- [ ] Listen on QUIC (`/ip4/0.0.0.0/udp/0/quic-v1`) + TCP; publish PeerId and
      all listen addrs into the roster; dial every new roster entry
- [ ] `snapshot.rs`: `{ player_id, x, y, vx, vy, tick, sent_at_ms }`, bincode,
      length-prefixed (4-byte BE)
- [ ] `send_snapshot.rs` on 30 Hz `FixedUpdate`, local box only. Drop stale
      outbound snapshots rather than queueing — latest-wins
- [ ] `apply_snapshot.rs`: discard snapshots with an older `tick` than the last
      applied (reordering); interpolate toward the target over ~100 ms rather
      than snapping
- [ ] Remote boxes = `RigidBody::Kinematic`
- [ ] **Checkpoint: two windows on one machine, each box moves in the other's
      window in real time.**

### M4 — Cross-network via a self-hosted relay pool (the actual goal)
- [ ] `--relay AUTO|YES|NO` CLI flag (default `AUTO`), parsed in `src/cli/`
      alongside the existing `cli_parse_*.rs` files
- [ ] `autonat` client: determine own reachability (`Public` / `Private`)
- [ ] **Relay server side**: if `YES`, or `AUTO` and AutoNAT says `Public`,
      enable `relay::Behaviour` (server) and register into a `relays` contract:
      `BTreeMap<PeerId, RelayEntry { addrs, updated_at, slots }>`, same
      commutative-merge rules as the roster. Heartbeat `updated_at`; readers
      ignore entries older than a TTL so dead relays age out.
      `AUTO` + `Private` ⇒ silently do not advertise (this is the common case)
- [ ] Leave a `relay_requirements.rs` seam — today it is only
      `reachability == Public`; later add CPU/bandwidth/uptime checks
- [ ] **Relay client side**: if `Private`, read the `relays` contract, reserve
      a slot on 1–2 live relays, publish the resulting `/p2p-circuit` addrs
      into the roster entry
- [ ] Dial peers over the relay first (always works), then let `dcutr` attempt
      the upgrade to direct QUIC
- [ ] Failure path: zero live relays in the contract → clear UI message, and
      still allow direct dial (works if either side is public or on the LAN)
- [ ] Connection-quality state machine: `Relayed → Direct` with the
      `dcutr::Event::DirectConnectionUpgradeSucceeded/Failed` transitions;
      surface both in the UI. Failure is expected ~30% of the time
- [ ] Measure and display RTT (`ping` + `sent_at_ms`) in both modes
- [ ] **Checkpoint: two machines on two different home networks, both boxes
      moving. Record which of relayed/direct each pair got.**

### M5 — Polish
- [ ] Player-name / connection-status UI (port `status_bubble` + message log
      from example_2 — without `bevy_tweening`)
- [ ] Handle disconnect: despawn the box, drop the roster entry
- [ ] ed25519 identity bridge: derive the libp2p `Keypair` from the same
      32-byte secret as the Freenet node identity
      (`Keypair::ed25519_from_bytes` → `with_existing_identity`), so a peer's
      roster entry is cryptographically bound to its `PeerId`
- [ ] e2e test: spawn two release binaries, assert both report "peer connected"
      (steal `ProcessOrchestrator`'s spawn-and-scrape-stdout approach from
      `libp2p_bevy_example/src/p2p/testing/`)
- [ ] `pre-push`: build --all-targets, clippy -D warnings, fmt --check,
      test --all-targets, `cargo run --manifest-path ../lele_lint/Cargo.toml`

## Explicitly out of scope

- Kademlia / DHT peer discovery — the Freenet contract is the rendezvous, we
  always know exactly which PeerId to dial
- Gossipsub — full mesh at 2–4 players; direct streams are lower latency
- mDNS — LAN-only discovery is a subset of what the roster already gives us
- Deterministic/lockstep physics, rollback, client prediction
- The CLI plugin — `BEVY_CLI_GOAL.md` says the example_2 `read_stdin` approach
  is what to avoid, and it blocks the Update loop. GUI only.
- More than ~4 concurrent players (full mesh is fine at that size)
- WASM/browser build

## Open questions

- **Bootstrapping the relay pool in practice.** If every player is behind a
  NAT, the `relays` contract is empty and nobody can connect. For testing,
  someone must run `--relay YES` on a reachable host (port-forwarded box,
  cheap VPS, or a machine on the same LAN as one tester). Worth deciding who
  before M4 starts.
- **Relay abuse / capacity.** `relay::Config` caps reservations and bandwidth;
  pick limits before anyone runs `--relay YES` on a real connection.
- **Should the relay list live in the roster contract or its own?** Separate
  contract is cleaner (different TTL, different write frequency, and a relay
  is not necessarily a player) — plan assumes separate.
- ~~Runtime topology~~ — answered by M0.5: one tokio runtime is enough.
- ~~UDP port conflict between freenet and QUIC~~ — answered by M0.5: none.
- **Does the Freenet node expose a ring-size / connected-peers query?** M0.5
  proved we need one (a Get response is not readiness) but did not find one.
  If there is no such API, the fallback is "retry the roster read until either
  non-empty or a floor of N seconds has passed".
- Roster entry expiry: TTL-based, or explicit leave message? TTL is more robust
  against crashes but needs a clock the contract can trust.

## Reference

- avian2d/avian3d **0.7** ↔ bevy **0.19** (0.18 → 0.5–0.6). Crate is
  `avianphysics/avian`; `full360systems/bevy_avian` is a fork, don't use it.
- DCUtR hole-punch success rate: **70% ± 7.1%**, TCP ≈ QUIC, 97.6% of
  successes on the first attempt — Trautwein et al., arXiv:2510.27500.

## Prior art to mine (not depend on)

- `libp2p_bevy_example/src/boxes/` — `character_controller`, `sync_position`,
  tick-based input broadcast, join/leave. Bevy 0.18 + old PascalCase
  convention, so port the design, rewrite the code.
- `libp2p_bevy_example/src/p2p/testing/ProcessOrchestrator` — multi-process
  test harness via `duct` + stdout scraping.
- `freenet_bevy_example_2/src/freenet/` — websocket client, reusable verbatim.
