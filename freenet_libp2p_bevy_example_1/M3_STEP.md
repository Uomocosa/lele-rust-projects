# M3 — libp2p real-time sync (same-machine / LAN first)

## Context

`freenet_libp2p_bevy_example_1` is a hybrid Freenet+libp2p Bevy game
(`TODO.md` in that directory). M0, M0.5, M1, and M2 are all done. M2 delivered
the Freenet-backed player roster: peers merge `PeerEntry { peer_id, addrs,
updated_at }` via a commutative-monoid contract, and a live two-node test
(`testing/tests/two_node_roster.rs`) proves two embedded nodes join and each
see a 2-entry roster. **M3 is the second half of the stack**: the libp2p swarm
that carries box transforms in real time. Roster = who is here and how to dial
them; libp2p = the actual movement data.

M0.5 already settled the runtime topology: **one ambient `#[tokio::main]`
runtime is enough** — freenet and the swarm both spawn onto it, no dedicated
thread and no nested `Runtime::new()` needed for correctness. The threading
decision for this milestone (see §7) is therefore `tokio::spawn` on that
ambient runtime, not a std thread, consistent with the existing
`roster::connect_and_run` and with the TODO's "own the `!Sync` swarm on a
single task" requirement.

Research against the verified libp2p 0.56 API surface (registry sources) plus
a close read of this project's current `src/` surfaced **three pre-existing
integration bugs** that M3 must fix before the new code can work, and the exact
builder/codec signatures the new code will use. All API facts below were
verified against the locked versions (`libp2p 0.56.0`,
`libp2p-request-response 0.29.0`, `libp2p-autonat 0.15.0`, `libp2p-dcutr
0.14.1`, `libp2p-identify 0.47.0`, `libp2p-ping 0.47.0`).

### Pre-existing bugs this milestone must fix

1. **Local box has the wrong `PlayerId`.** `main.rs:19-24` computes `own_id`
   from unix-nanos, but `src/boxes/bevy_systems/setup.rs:21` spawns the local
   box with the hardcoded `PlayerId(0)`. Since `spawn_roster_boxes` spawns a
   box for every roster entry whose id is not already in the world, the nanos
   `own_id` (which is always in the roster, merged by `setup_contract`) gets a
   **duplicate, non-local box**. Fix: the local box must carry the real
   `own_id`, so the roster spawner skips it (it is already in the spawned set).
2. **Roster `peer_id`/`addrs` are placeholders, not dial info.** `main.rs:25-29`
   publishes `peer_id = "player-{id}"` and `addrs = []`. M3 needs the swarm's
   actual `PeerId` and listen multiaddrs in the entry so the *other* peer can
   dial. The swarm must be listening (and its first `Ready` event emitted)
   **before** the roster entry is built and put on the contract.
3. **Remote boxes are `RigidBody::Dynamic`.** `spawn_box.rs:18` always uses
   `Dynamic`; the TODO's non-negotiable constraint says remote boxes must be
   `RigidBody::Kinematic` and be driven purely by incoming snapshots (peer
   authoritative over its own box only). `spawn_box` already takes `is_local` —
   the body type must follow it.

### Verified libp2p 0.56 API facts (locked versions)

- **Builder chain** (TODO M3, confirmed present in `builder.rs` and the phase
  files): `SwarmBuilder::with_new_identity()?.with_tokio()?.with_tcp(tcp::Config::default(), noise::Config::new, yamux::Config::default)?.with_quic()?.with_dns()?.with_relay_client(noise::Config::new, yamux::Config::default)?.with_behaviour(..)?.with_swarm_config(..).build()`. `with_relay_client` returns a phase that hands `with_behaviour` the relay client behaviour: the closure is
  `FnOnce(&Keypair, libp2p_relay::client::Behaviour) -> R`
  (`builder/phase/behaviour.rs:17`). So the behaviour constructor receives
  both the identity keypair and the pre-built `relay::client::Behaviour`.
- **request-response** (`with_codec` at `lib.rs:395`):
  `request_response::Behaviour::with_codec(codec, protocols, config)` where
  `protocols` is an iterator of `(StreamProtocol, ProtocolSupport)` — use
  `ProtocolSupport::Full` (both directions) per the TODO. The `Codec` trait is
  async-trait (`read_request`/`read_response`/`write_request`/
  `write_response`), `Protocol: AsRef<str>`, `Request`/`Response: Debug + Send`.
  Inbound requests arrive as
  `Event::Message { peer, message: Message::Request { request, channel } }`;
  outbound answers as `Message::Response { request_id, response }`. Reply via
  `swarm.behaviour_mut().positions.send_response(channel, response)`.
  `StreamProtocol` implements `AsRef<str>`.
- **identify**: `identify::Config::new(protocol_version: String,
  local_public_key: PublicKey)` (`behaviour.rs:166`) or
  `new_with_signed_peer_record(protocol_version, &keypair)` (`:174`), wrapped in
  `identify::Behaviour::new(config)` (`:261`).
- **ping**: `ping::Behaviour::new(config)` (`ping/src/lib.rs:98`), `Default`
  impl exists; set an interval for RTT.
- **autonat**: `autonat::Behaviour::new(local_peer_id, config)`
  (`v1/behaviour.rs:224`), re-exported at the crate root (`pub use v1::*`).
- **dcutr**: `dcutr::Behaviour::new(local_peer_id)`.
- **relay client**: with the builder chain above, the behaviour is supplied by
  `with_relay_client`; the standalone constructor
  `relay::client::new(local_peer_id) -> (Transport, Behaviour)` exists but is
  **not** needed with the builder. Wired now, exercised in M4.
- **Dependencies already present** in `Cargo.toml`: `libp2p 0.56` with all
  needed features, `tokio` (rt, sync, macros, rt-multi-thread, time), `futures`
  + `futures-util`, `serde`, `bincode`, `thiserror`, `tracing`. **Missing:**
  `async-trait` (the request-response `Codec` trait requires it) and the
  `tokio-stream` crate (to adapt `UnboundedReceiver` into a `Stream` for the
  swarm loop's `select!`; alternatively use `tokio::select!` with
  `cmd_rx.recv()` directly — no stream adapter needed).

## Plan

### 1. `src/p2p/` domain — swarm, behaviour, codec, threading bridge

New `src/p2p/` domain folder following the project's atomic-file + thin
delegate conventions (`mod.rs` stays a pure module tree per lele E019):
- `snapshot.rs` — `Snapshot { player_id: u64, x: f32, y: f32, vx: f32, vy:
  f32, tick: u64, sent_at_ms: u64 }`, `derive(Debug, Clone, Copy,
  Serialize, Deserialize)`. This is the wire format, matching the TODO spec.
- `codec.rs` — `SnapshotCodec` implementing `request_response::Codec`:
  `Protocol = StreamProtocol`, `Request = Snapshot`, `Response = Snapshot`,
  `AsyncRead/AsyncWrite` length-prefixed (4-byte BE) bincode framing. Add
  `test_usage`: encode+decode roundtrip through an in-memory pipe.
- `behaviour.rs` + `behaviour_new.rs` — `#[derive(NetworkBehaviour)]
  struct Behaviour { positions: request_response::Behaviour<SnapshotCodec>,
  identify, ping, autonat, dcutr, relay: relay::client::Behaviour }`.
  Constructor free fn `behaviour_new(key: &Keypair, relay) -> Behaviour` using
  the verified `with_codec`/`Config`/`new` signatures above. `#[behaviour(event_process = false)]` so events surface as `BehaviourEvent`.
- `swarm.rs` — `build_swarm() -> Result<Swarm<Behaviour>, Error>` using the
  verified builder chain (identity → tokio → tcp(noise,yamux) → quic → dns →
  relay_client → behaviour → config). Listen on `/ip4/0.0.0.0/udp/0/quic-v1`
  **and** `/ip4/0.0.0.0/tcp/0`. Set `SwarmConfig::with_idle_connection_timeout`
  generously (default 10 s is shorter than game pauses and would drop the
  connection between snapshot bursts).
- `command.rs` — `enum Command { Dial { peer_id: String, addrs: Vec<String> },
  SendSnapshot { peer_id: String, snapshot: Snapshot } }` (Bevy → swarm task).
- `event.rs` — `enum Event { Ready { peer_id: String, addrs: Vec<String> },
  PeerConnected(PeerId), PeerDisconnected(PeerId), IncomingSnapshot { from:
  PeerId, snapshot: Snapshot }, Error(String) }` (swarm task → Bevy).
- `swarm_thread.rs` — the owned-Swarm loop:
  - construct the swarm via `build_swarm()`, emit `Event::Ready` with the
    local `PeerId.to_base58()` and all listen addrs once the first
    `NewListenAddr` arrives;
  - `tokio::select!` over `swarm.select_next_some()` (futures `StreamExt`) and
    `cmd_rx.recv()` (`tokio::select!` handles the receiver directly — no
    `tokio-stream` needed);
  - on `Command::Dial` → `swarm.dial` each `/p2p/`-suffixed multiaddr (skip
    failures with a log + `Event::Error`);
  - on `Command::SendSnapshot` → `send_request(peer_id, snapshot)`;
  - on `BehaviourEvent::Positions(Event::Message { message:
    Message::Request { request: snapshot, channel } })` → record the incoming
    snapshot and **reply with our own current snapshot** via `send_response` —
    one request/response round trip carries both directions;
  - forward `ConnectionEstablished`/`ConnectionClosed` and outbound-response
    messages as the corresponding `Event`s.
  `// no test_usage necessary` — exercised via the §8 `#[tokio::test]`.
- `command_tx.rs` / `event_rx.rs` resources — `#[derive(Resource, Deref)]`
  `P2pCommands(UnboundedSender<Command>)` and `P2pEvents` (`Deref, DerefMut`,
  mirroring `roster::RosterEvents`).
- `config.rs` + `config_new.rs` — `Config { cmd_tx, event_rx }` (named fields,
  E018) with a `new(cmd_tx, event_rx)` constructor. Bevy holds `cmd_tx` to send
  commands; the plugin consumes `event_rx` into the resource.
- `plugin.rs` + `plugin_build.rs` — delegate pair mirroring `roster::Plugin`.
  `pub struct Plugin(pub p2p::Config)` (single field → newtype + `Deref`,
  E018). `plugin_build` inserts `P2pCommands` (clone of `cmd_tx`) and
  `P2pEvents` (taken `event_rx`), registers the systems from §5.
- `constants.rs` — protocol id `/freenet-boxes/positions/1.0.0`, snapshot
  rate 30 Hz, interpolation window ~100 ms, idle connection timeout.
- `error.rs` — `thiserror` enum: `Build(String)`, `Dial(String)`,
  `Swarm(String)`. No `unwrap`/`expect`/`panic`.

### 2. `boxes` fixes (the three pre-existing bugs)

- `boxes::Config` — new single-field newtype `Config(pub PlayerId)`
  (`derive(Deref)`, E018) so the domain knows the real local id. Insert it as
  a `Resource` in `boxes::plugin_build`, and make `bevy_systems/setup.rs` read
  `Res<boxes::Config>` and spawn the local box with **that** `PlayerId`
  instead of `PlayerId(0)`. This kills the duplicate-box bug and makes the
  local box's id line up with the roster key it publishes.
- `spawn_box.rs` — pick the body type from `is_local`:
  `RigidBody::Dynamic` when local, `RigidBody::Kinematic` when remote. Update
  the existing `test_usage` to assert a remote spawn carries `Kinematic` and no
  `LocalPlayer` marker.
- `boxes::Plugin` — change from `pub struct Plugin;` to a tuple newtype
  carrying the config so `main.rs` can hand it the own id
  (`Plugin(boxes::Config::new(own_id))`), matching the `roster::Plugin`
  shape.

### 3. `src/cli/` domain — `--p2p-port` (ported, minimal)

The M4 TODO already expects a `src/cli/` folder ("alongside the existing
`cli_parse_*.rs` files"), and the two-instance checkpoint needs each process to
bind a distinct Freenet UDP port (M2's `start_embedded_node` takes `p2p_port`;
two instances both defaulting would collide). Port the minimal
`--p2p-port` arg parser from `freenet_bevy_example_2/src/cli/cli_parse_p2p_port.rs`
(parse the flag, else probe a free UDP port). Pass the result through
`roster::connect_and_run` (its `p2p_port` parameter already exists — `main.rs`
currently passes `0`).

### 4. `main.rs` wiring

Order matters (bug #2): the swarm must be listening before the roster entry is
published.
1. Create the p2p channels: `(cmd_tx, cmd_rx)` and `(event_tx, event_rx)`
   (`tokio::sync::mpsc::unbounded_channel`).
2. `tokio::spawn(p2p::swarm_thread::run(cmd_rx, event_tx))`.
3. Await the first `Event::Ready { peer_id, addrs }` on `event_rx`.
4. Build `own_entry` from it: `peer_id` = the base58 `PeerId`, `addrs` = the
   listen multiaddrs (each with `/p2p/<peer_id>` appended, so the dialer
   verifies the peer id on connect), `updated_at = now`.
5. `tokio::spawn(roster::connect_and_run(p2p_port, contract_wasm, own_id,
   own_entry, roster_tx))` (unchanged except the port now comes from the CLI).
6. Register plugins: `boxes::Plugin(boxes::Config::new(own_id))`,
   `roster::Plugin(...)`, and `p2p::Plugin(p2p::Config::new(cmd_tx, event_rx))`.

### 5. Bevy systems (`src/p2p/bevy_systems/`)

All drain/act with `try_recv()` — **nothing on the Bevy schedule blocks**
(TODO constraint):
- `poll_swarm_events.rs` — fully drain `P2pEvents` each `Update`; on
  `IncomingSnapshot` write a `RemoteTarget { pos: Vec2, tick: u64 }` component
  onto the box matching `PlayerId` (discard if `tick <=` last applied for that
  player — reordering guard); on `PeerConnected`/`PeerDisconnected` update a
  connection-status resource for the UI later.
- `dial_roster_peers.rs` — on `Roster` change, for every entry whose
  `peer_id` is not self and not in a `DialedPeers` set, send
  `Command::Dial { peer_id, addrs }` (tries every candidate addr, including the
  eventual `/p2p-circuit` relay addrs in M4).
- `send_snapshot.rs` — on 30 Hz `FixedUpdate` (`Time::<Fixed>`), read the
  local box's `Position`/`LinearVelocity`, build a `Snapshot` with a monotonic
  `tick`, and send `Command::SendSnapshot` to **every known remote roster
  entry**. Latest-wins: only the current state is ever sent, stale pending
  commands are simply not queued (the mpsc is unbounded but the loop never
  enqueues backlog).
- `interpolate_remote_boxes.rs` — on `FixedUpdate`, for each remote box with a
  `RemoteTarget`, lerp `Position` toward the target over ~100 ms instead of
  snapping (TODO requirement); remove the target once reached.

### 6. Constants and the 30 Hz budget

`p2p/constants.rs` holds `PROTOCOL_NAME`, `SNAPSHOT_HZ = 30`,
`INTERPOLATION_WINDOW = Duration::from_millis(100)`. The 30 Hz FixedUpdate
pipeline is: `send_snapshot` → `poll_swarm_events` → `interpolate_remote_boxes`
(chained in the plugin). Local input/physics is untouched; the local box stays
`Dynamic`, the remote boxes are `Kinematic` and driven solely by snapshots.

### 7. Threading model (decided)

`tokio::spawn` on the ambient `#[tokio::main]` runtime — the same runtime
freenet already uses (`connect_and_run`), validated by M0.5 (one runtime is
enough; QUIC and freenet's UDP coexist). The `!Sync` `Swarm` is owned entirely
inside the spawned task and polled via `select_next_some()` in a tight loop;
Bevy communicates only over the mpsc channels (`try_send`/`try_recv`). No
`blocking_recv`, no `lock().unwrap()` round trips into the swarm task (the
`libp2p_bevy_example` anti-pattern the TODO calls out).

### 8. Tests & verification

- **Unit (in-file `test_usage`, per lele):** `snapshot` bincode roundtrip;
  `codec` length-prefix roundtrip through an in-memory `AsyncRead/AsyncWrite`
  pipe; `behaviour_new` constructs with the builder's closure shape; updated
  `boxes` tests (local `Dynamic` + `LocalPlayer`, remote `Kinematic`).
- **`#[tokio::test]` in `swarm_thread.rs`** (or a dedicated test file): build
  **two swarms in-process**, connect them over loopback, have A `send_request`
  a snapshot to B, assert B's loop emits `IncomingSnapshot` with the same data
  and B's reply arrives back at A. Proves behaviour + codec + threading bridge
  end-to-end without freenet or a live ring. (Two swarms on one ambient runtime
  is exactly the M0.5-proven topology.)
- **Headless `App` tests:** `send_snapshot` with a fake roster produces a
  `Command::SendSnapshot`; `poll_swarm_events` fed an `IncomingSnapshot` moves
  a pre-spawned `Kinematic` box's `RemoteTarget`; `dial_roster_peers` emits one
  `Dial` per new remote entry and none for self.
- **Manual checkpoint (the M3 acceptance test):** run two `freenet-libp2p-bevy-example-1`
  binaries with distinct `--p2p-port` values, let them join the public ring,
  each finds the other via the roster, dials over loopback QUIC/TCP, and each
  box moves in the other's window in real time. Verified visually via the
  desktop-control tooling (spawn + screenshot both windows), same as M1/M2.
- **Standard routine:** `cargo build --all-targets`, `cargo clippy -- -D
  warnings`, `cargo fmt -- --check`, `cargo test --all-targets`,
  `cargo run --manifest-path ../lele_lint/Cargo.toml`. (`freenet` builds need
  the target-dir redirect already in `.cargo/config.toml`.)

## Open questions (carried from TODO, not blocking M3)

- Roster entry expiry / TTL (only matters once entries outlive their player).
- Relay pool bootstrapping and the `--relay` flag live in M4, not here — this
  milestone only wires `relay::client` into the behaviour and the builder so
  M4's closure signature is already stable.

## Reference

- `TODO.md` M3 milestone checklist and Constraints section (this plan maps
  directly onto it).
- Prior art: `freenet_libp2p_example/` (M0.5 spike — swarm loop on the ambient
  runtime), `freenet_bevy_example_2/src/cli/cli_parse_p2p_port.rs` (port
  parsing), `libp2p_bevy_example/` (design to port, not code — its blocking
  `get_connected_peers` is the anti-pattern).
