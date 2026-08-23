# Plan: `freenet_libp2p_bevy_plugin` generic plugin + `freenet_libp2p_bevy_plugin_games`

## Decisions (confirmed)
- **Plugin payload is generic** over a game message type `T` — one plugin serves both games.
- **Games** live in `freenet_libp2p_bevy_plugin_games/` as **standalone Cargo crates** (`boxes_game/`, `clicker_game/`), each with a path dependency on the plugin.
- **`freenet_libp2p_bevy_example_1` stays frozen** — used only as the blueprint; no edits.
- **Per-game contracts**: each game crate owns and embeds its own `contract/` freenet WASM subcrate. The plugin ships only the generic roster-contract runtime (host side); games plug their membership contract bytes into roster config, and game state sync flows over the generic p2p `T`. No wire compatibility across games is required.

---

## A. Plugin crate `freenet_libp2p_bevy_plugin/`

Refactor the current `freenet_libp2p_bevy_plugin` (now `build.rs` + `contract/` + `.cargo/config.toml`) into a real Rust library crate. Its core thesis: *the network layer is game-agnostic; it never spawns/despawns/interpolates game entities — it only emits events and accepts commands.*

### 1. Cargo & harness
- New `Cargo.toml`: `[workspace] members=[".", "contract"]` added via **exclude** (so `contract` is not part of the library's dep graph), package `freenet_libp2p_bevy_plugin`, edition 2024, `[lib] name = "freenet_libp2p_bevy_plugin_lib"`, `crate-type=["lib","cdylib"]`. Same pinned deps as example_1 (bevy =0.19, avian2d, freenet =0.2.128, freenet-stdlib net, libp2p =0.56 ...same feature set, tokio/tungstenite, thiserror, tracing, serde, bincode, derive_more deref/deref_mut).
- Keep `build.rs` and `contract/roster_contract.wasm` exactly as-is (the contract mirrors `example_1`'s roster contract verbatim, keyed by a `u64` id — already neutral).
- Keep `.cargo/config.toml` (target-dir to `/home/uomocosa/.cache/frt-build`, mold linker) so the space-in-path build works.

### 2. Module tree (atomic files, lele conventions)
```
src/
  lib.rs                        # pub mod cli; pub mod freenet; pub mod net_id; pub mod p2p; pub mod plugin; pub mod roster;
  net_id.rs                     # pub struct NetworkId(pub u64) #[derive(Deref)]  ← replaces boxes::PlayerId
  cli/                          # ported verbatim from example_1 (fully generic: identity_dir, contract_params, freenet_local, freenet_gateway)
  freenet/                      # ported verbatim (fully decoupled WebSocket client + FreenetConnectionError)
  plugin.rs                     # pub struct Plugin<T>(Config<T>) tuple-newtype + Deref; thin delegate to plugin_build
  plugin_build.rs               # wires roster+p2p+freenet resources & systems for P2pPlugin<T>
  roster/                       # membership sync, neutral over NetworkId  (game systems REMOVED)
  p2p/                          # generic transport over T  (bevy_systems made generic or replaced by events)
```
Follow example_1's atomic-file naming exactly (`*_<method>.rs`, private method modules, thin delegates with `#[rustfmt::skip]`, `mod.rs` only `mod/pub mod/pub use`).

### 3. `net_id.rs`
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deref)]
pub struct NetworkId(pub u64);
```
Mapped from `PeerId` via the existing FNV-1a `peer_id_to_player_id` logic (ported, return type becomes `NetworkId`). `NetworkId` is the only id type the plugin knows.

### 4. Generify `p2p/` over `T`
- `message.rs`: `pub trait Message: Serialize + Deserialize + Send + Sync + 'static {}`
- `snapshot.rs`: `Snapshot<T> { from_id: NetworkId, tick: u64, sent_at_ms: u64, payload: T }` (serde/bincode, `T: Message`).
- `codec.rs` (`snapshot_codec` → `message_codec<T>`): `MessageCodec<T>` implementing `request_response::Codec` with `Protocol=StreamProtocol`, `Request/Response=Snapshot<T>`.
- `command.rs`: `Command<T> { Dial { peer_id, addrs }, SendSnapshot { peer_id, snapshot: Snapshot<T> } }`.
- `event.rs`: `Event<T> { Ready{peer_id,addrs}, PeerConnected(PeerId), PeerDisconnected(PeerId), IncomingSnapshot{from:PeerId,snapshot:Snapshot<T>}, Error(String) }`.
- `behaviour.rs` / `behaviour_new.rs` / `run.rs` / `build_swarm.rs` / `snapshot_tick.rs` / `config.rs(+_new/_take_event_rx)` / `load_or_create_keypair.rs` / `dialed_peers.rs` / `peer_status.rs` / `remote_target.rs` / `p2p_events.rs` / `error.rs`: **genericized over `T: Message`** end-to-end (mechanical type-param threading; `RemoteTarget` stays a neutral position component the plugin still owns, or moves game-side — see hook note).
- **Remove** the game-coupled systems `interpolate_remote_boxes`, `send_snapshot`, `poll_swarm_events`, `dial_roster_peers` from the plugin, and instead:
  - Poll `P2pEvents<T>` and emit **Bevy events** `P2pReady { own: NetworkId }`, `PeerConnected { id: NetworkId }`, `PeerDisconnected { id: NetworkId }`, and spawn component `RemoteEntity { id: NetworkId, pending: Snapshot<T>, interpolate: bool }` — OR simpler: expose `IncomingSnapshot<T>` as a Bevy event + a `SendCommand<T>` resource. **Games own interpolation/spawn/despawn.** This is the decoupling that makes the plugin reusable.

### 5. Neutral `roster/`
Port from example_1: `PeerEntry`, `FreenetStatus`, `NodeInfo`, `RosterEvents`, `Roster`, `RosterState`, `Event`, `roster_digest`, `prune_stale`, `merge_roster`, `decode_roster_update`, `setup_contract`, `connect_client_loop`, `start_embedded_node`, `connect_and_run`, `connect_*_args`, `config(+_new/_take_event_rx)`, `constants`, `peer`.
- Replace every `boxes::PlayerId` key with `network_id: NetworkId`. `Roster` = `BTreeMap<NetworkId, PeerEntry>`.
- **Remove** `spawn_roster_boxes` / `despawn_roster_boxes` (game-coupled). Emit Bevy events `PeerJoin { id: NetworkId }` / `PeerLeave { id: NetworkId }` from `poll_freenet_events`.
- `connect_and_run_args` / `connect_client_args` `own_id` becomes `NetworkId`; `contract_wasm` stays an **input** (per-game contract bytes injected by the game's `main.rs`).

### 6. `plugin.rs` / `plugin_build.rs`
- `pub struct Plugin<T: Message>(Config<T>)` with `#[derive(Deref)]`, thin delegate.
- `Config<T>` injects: `own_id: NetworkId`, `cmd_tx`/`event_rx` (p2p), `roster_rx` (+ `contract_wasm`, freenet node args). Constructed by the game's `main.rs` from its own `cli::Cli` + embedded contract, exactly mirroring example_1's `main.rs:28-72`.
- `plugin_build::build` registers resources `P2pCommands<T>`, `P2pEvents<T>`, `Roster`, `RosterEvents`, `FreenetStatus`, `SnapshotTick`, `Time::<Fixed>` and systems; emits `PeerJoin`/`PeerLeave`/`PeerConnected`/`PeerDisconnected`/`IncomingSnapshot<T>` Bevy events and provides `SendCommand<T>` for games to request dials/snapshots. No game entities are touched.

---

## B. `freenet_libp2p_bevy_plugin_games/`

### 7. `boxes_game/` (position game, modeled on example_1's `boxes`)
- Own Cargo workspace (`members=[".", "contract"]` excluded). Path dep: `freenet_libp2p_bevy_plugin = { path = "../freenet_libp2p_bevy_plugin" }`. Own `build.rs` + `contract/` (per-game roster membership contract).
- Port `boxes/` module from example_1: `Player`, `LocalPlayer`, `Config`, `spawn_box`, `move_box`, `jump_box`, `pick_spawn_x`, `spawn_x_for_player`, physics setup, constants — **replace `boxes::PlayerId` with `NetworkId`**.
- **`T = Position { id: NetworkId, x: f32, y: f32, vx: f32, vy: f32 }`** (implements `Message`).
- Game glue systems (consume plugin events → drive boxes): on `PeerJoin` spawn a remote box; on `PeerConnected` start sending `<Position>` snapshots via `SendCommand<T>`; on `IncomingSnapshot<T>` interpolate the remote box transform; on `PeerLeave`/`PeerDisconnected` despawn.
- `src/main.rs` mirrors example_1 `main.rs` (cli, keypair, `p2p::run`, `roster::connect_and_run`), injects per-game contract bytes, then `app.add_plugins(freenet_libp2p_bevy_plugin::Plugin::new(...)).add_plugins(boxes::Plugin)` etc.

### 8. `clicker_game/` (counter game, modeled on bevy clicker)
- Own Cargo workspace, path dep on the plugin, own `build.rs` + **`contract/` clicker contract** syncing counter aggregate via freenet (per-game contract).
- Port the clicker logic from `bevy_libp2p_1` (`ClickCounter`, `ClickTarget`, `Owner`, `detect_click`, `update_counter`, join/leave handling), **rewired to the plugin**:
  - `Owner(NetworkId)` instead of `PeerId`.
  - On plugin `PeerJoin` → spawn a per-peer counter entity; on `PeerLeave` → despawn.
  - **`T = ClickDelta { clicks: u32 }`** real-time broadcast over p2p on each click through `SendCommand<T>`; `IncomingSnapshot<T>` updates the opponents' counters.
  - The freenet **clicker contract** (per-game) persists/aggregates counts for later joiners (catch-up), demonstrating per-game contract authoring.
- `src/main.rs` analogous to boxes_game, injecting the clicker contract.

---

## C. Verification
For each of the plugin and both game crates (with **`CARGO_TARGET_DIR=/tmp/frt-build`** prepended due to spaces in path):
```bash
cargo build --all-targets
cargo clippy -- -D warnings
cargo fmt -- --check
cargo test --all-targets
cargo run --manifest-path ../lele_lint/Cargo.toml
```
- Every `src/` module gets a `test_usage` test; thin-delegate modules may note `// no test_usage necessary` per lele conventions.
- Adhere to lele_lint E-codes: atomic files (E001/E002), method-file privacy (E003), `test_usage` (E006), no positional access / E018 tuple newtype vs named-field rule (NetworkId = single-field tuple newtype + `Deref`; multi-field structs use named fields), `mod.rs` purity (E019), no `crate::` outside `use` (E020), domain-prefix imports (E011), thiserror everywhere (no unwrap/expect/panic).

## D. Optional follow-ups (out of scope unless you want them)
- `opencode.json` per-project skill filtering for the new crates.
- CI/crate-tag support for plugin + games (via `test-orchestrator`), reusing the example_1 patterns.

---

### Key assumption
"**Per-game contracts**" is implemented as: each game embeds its **own** roster/clicker contract WASM (its own `contract/` subcrate), so the plugin stays generic; the clicker contract also aggregates counts for catch-up. If instead the intent is that **all** game state (including boxes positions) is mirrored through freenet contracts (not just libp2p `T`), that is a scope extension to request explicitly.