# M2 — Roster contract on Freenet

## Context

`freenet_libp2p_bevy_example_1` is a hybrid Freenet+libp2p Bevy game
(`TODO.md` in that directory). M0 (scaffolding) and M1 (local physics game,
no networking) are both done. M2 is next: give the game a Freenet-backed
player roster — a commutative-monoid contract that peers merge locally so
each player can discover the others' `PeerId` and dial addresses, without a
central server. Nothing gets built on top of this until it works, since M3
(libp2p real-time sync) depends on peers being able to find each other via
the roster.

Research against the sibling project `freenet_bevy_example_2` (which has a
working single-purpose "clicker" contract, a reusable `src/freenet/`
websocket client, and a `testing/` harness) confirms most of the M2 plumbing
can be reused directly, with three real changes:
1. The clicker contract's `update_state` **replaces** the whole state; the
   roster contract's `update_state` must be a real merge.
2. `freenet_bevy_example_2` never actually joins two nodes together in a
   test — its `testing::TestNode` always starts an isolated single-node
   network. The two-node join test M2 wants is new work.
3. The existing "readiness" check (`testing/start.rs::probe_ready`, and the
   embedded node's hardcoded `sleep(20s)` in `main.rs`) is the exact false
   signal M0.5 flagged: "got *any* Get response, including `NotFound`" is
   not the same as "joined the ring." A real signal exists and was
   confirmed present in the exact `freenet` 0.2.123 version this project is
   locked to: `ClientRequest::NodeQueries(NodeQuery::NodeDiagnostics{
   config: NodeDiagnosticsConfig::basic_status() })`, answered with
   `QueryResponse::NodeDiagnostics(NodeDiagnosticsResponse)` containing
   `network_info.active_connections` and `node_info`. This is reachable over
   the same websocket connection `FreenetClient` already uses — no new
   dependency.

## Plan

### 1. `contract/` crate — the roster contract
New crate at `freenet_libp2p_bevy_example_1/contract/`, `Cargo.toml`
targeting `wasm32-unknown-unknown` (mirror
`freenet_bevy_example_2/contract/Cargo.toml` for the freenet-stdlib dep and
crate-type). `contract/src/lib.rs`:
- `PlayerId` (already exists as a Bevy `Component` field type in
  `src/boxes/player_id.rs` — the contract crate is compiled to WASM
  separately and cannot depend on the `freenet_libp2p_bevy_example_1_lib` lib, so it needs its
  own `PlayerId` newtype, `derive(Ord, PartialOrd, Serialize, Deserialize)`
  for the `BTreeMap` key)
- `PeerEntry { peer_id: String, addrs: Vec<String>, updated_at: u64 }`
- `RosterState = BTreeMap<PlayerId, PeerEntry>`, bincode-serialized
- `ContractInterface` impl on `RosterContract` (pattern from
  `freenet_bevy_example_2/contract/src/lib.rs`):
  - `validate_state`: deserialize, `ValidateResult::Valid` / `InvalidState`
  - `update_state`: **the real change** — iterate *every* `UpdateData` item
    (not just the first, unlike the clicker contract), union all keys
    across the current state and every update, and per key keep the entry
    with the larger `updated_at`
  - `summarize_state` / `get_state_delta`: return the full state (mirrors
    the clicker contract's approach); add a regression test like the
    clicker contract's `test_get_state_delta_carries_the_count` guarding
    against ever returning an empty delta
  - Unit tests (no wasm runtime needed, direct static calls per the clicker
    contract's test pattern): commutativity (`merge(A,B) == merge(B,A)`),
    idempotence (`merge(A,A) == A`), associativity
    (`merge(merge(A,B),C) == merge(A,merge(B,C))`)

### 2. Build wiring (deferred in M0)
- Add `build.rs` at the project root, copied from
  `freenet_bevy_example_2/build.rs` (rebuilds `contract/` to wasm and
  copies `contract/roster_contract.wasm` into the repo root on staleness)
- Restore `build-contract` / `copy-wasm` tasks in `Makefile.toml` (dropped
  in M0 because no contract existed yet), and make `test` / `pre-push`
  depend on `copy-wasm` again, matching example_2's `Makefile.toml`
- `TODO.md`/`OBJECTIVE.md` already flagged this deferral — check it off

### 3. `Cargo.toml` — add missing deps for `src/freenet/*`
Add `futures-util = "0.3"` (the existing `futures = "0.3"` umbrella crate is
not enough — `freenet_client_connect.rs` imports `futures_util::{SinkExt,
StreamExt}` directly), `http = "1"`, `tempfile = "3"`. Everything else
(`bincode`, `thiserror`, `tracing`, `tokio-tungstenite`, `freenet-stdlib`
with `features = ["net"]`) is already present and version-matched.

### 4. `src/freenet/` — reuse verbatim
Copy all 9 files from `freenet_bevy_example_2/src/freenet/` (`mod.rs`,
`freenet_client.rs` + its 6 method files, `freenet_connection_error.rs`) as
listed in the M2 TODO — no logic changes expected, just confirm it compiles
against this project's `freenet`/`freenet-stdlib` versions (0.2.123 vs
example_2's 0.2.116 — same NodeDiagnostics API confirmed present in both).

### 5. Real readiness check
Replace the embedded-node startup's blind `sleep(20s)` (pattern currently
in `freenet_bevy_example_2/src/main.rs:256-303`) with a poll loop, added
alongside wherever this project spawns its embedded node: after connecting
a `FreenetClient`, repeatedly send
`ClientRequest::NodeQueries(NodeQuery::NodeDiagnostics{ config:
NodeDiagnosticsConfig::basic_status() })` on a short interval until
`node_info` is populated (lone gateway bootstrapped) or
`network_info.active_connections >= 1` (joining peer actually connected),
with a ceiling timeout as a fallback. This same real signal should replace
`testing::TestNode::start()`'s `probe_ready` (which currently accepts any
Get response, including `NotFound`, as "ready" — exactly the false-positive
M0.5 warned about).

### 6. `roster` domain — Bevy-side resource + polling system
New `src/roster/` domain (own folder, per the project's atomic
domain-per-folder convention, since this is networking/state, not physics):
- `roster.rs`: `#[derive(Resource)] Roster { entries: BTreeMap<PlayerId,
  PeerEntry> }` (mirrors the contract's state shape)
- `bevy_systems/poll_freenet_events.rs`: drains the `FreenetClient`'s
  receive channel **fully each frame** (`while let Ok(...) = try_recv()`),
  applying `UpdateNotification`s into the `Roster` resource — the M2 TODO
  explicitly calls out that example_2's `poll_freenet_events` incorrectly
  returns after a single event; do not copy that bug
- `plugin.rs` / `plugin_build.rs` delegate pair wiring the embedded node
  startup, the `FreenetClient`, the `Roster` resource, and the polling
  system, following the same thin-delegate pattern as `src/boxes/plugin.rs`

### 7. `testing/` crate + two-node integration test
Copy the `TestNode` harness structure from
`freenet_bevy_example_2/testing/` (`structs/test_node.rs`,
`methods/test_node/{start,start_at,shutdown,port}.rs`, the
`connect`/`deploy`/`load_wasm`/etc. helpers), update `probe_ready` to use
the real readiness check from step 5, and add the **new** two-node join
test the source project never had: start node A as `is_gateway: true`,
capture its public key/address, start node B with `NetworkArgs.gateway`
pointing at node A (`"ip:port,hex-pubkey"` format, confirmed present in
`freenet` 0.2.123's `NetworkArgs`), wait via the real readiness check on
both, then `Put`/`Get` the roster contract from both sides and assert each
sees a 2-entry roster. This step is the least mechanical part of M2 (no
existing pattern to copy) — treat it as its own checkpoint and expect to
iterate on the exact `NetworkArgs`/public-key plumbing during
implementation.

## Verification
- `cargo test -p contract` (or equivalent workspace path): the
  commutativity/idempotence/associativity/delta-non-empty unit tests pass
  without a wasm runtime
- `cargo build --all-targets` green in `freenet_libp2p_bevy_example_1/`,
  including the new `build.rs` wasm compile step
- `cargo test --all-targets` in the `testing/` crate: two `TestNode`s join
  (node B dials node A via gateway), both `Put`/`Get` the roster contract,
  both observe a 2-entry `RosterState` — this is the M2 checkpoint from
  `TODO.md`
- `cargo clippy --all-targets -- -D warnings` and `cargo fmt -- --check`
  clean, matching the project's standard verification routine
- Check off the completed M2 items in `TODO.md` as they land, same as was
  done for M0 and M1
