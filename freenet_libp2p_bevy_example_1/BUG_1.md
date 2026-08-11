# BUG_1: same-machine multi-instance sync is unreliable

## Summary

Running two or more `bevy_freenet` instances on the same machine is supposed
to let each player see and move the others' boxes in real time (this was the
M3 milestone's acceptance test — see `M3_STEP.md`). As of commit `b8128a0`
this broke deterministically; after the fix in `2ce6f3b` it's back to
working *most of the time*, but not reliably, and the residual failure mode
traces to an open upstream Freenet issue, not our code.

There are two separate causes tangled together here. Both are documented
below with evidence.

## Cause 1 (fixed in `2ce6f3b`): shared libp2p identity file, plus a race

### What was there before (`a5b55fa`, the last confirmed-working commit)

`src/p2p/build_swarm.rs` called `libp2p::SwarmBuilder::with_new_identity()`
— every `cargo run` got a **fresh random keypair**, every single launch, no
persistence. Two instances on the same machine trivially got distinct
libp2p `PeerId`s (and, via `p2p::derive_player_id`, distinct
`boxes::PlayerId`s), so the roster contract never collided and libp2p never
tried to dial itself. `M3_STEP.md`'s checkpoint records this as manually
verified: "each box moves in the other's window in real time."

### What changed (`b8128a0`)

`p2p::load_or_create_keypair()` was added to persist the identity to
`~/.local/share/bevy_freenet/identity.bin`, so a player's identity — and
their roster entry — survives across restarts (avoiding a stale duplicate
roster entry every relaunch; see `prune_stale.rs` added in the same
commit). Reasonable goal, but the path was derived only from `$HOME`, with
no way to distinguish two players on one machine.

**Confirmed via `git diff a5b55fa b8128a0`:** `src/roster/start_embedded_node.rs`
— the code that decides how the embedded Freenet node joins the network —
is byte-for-byte unchanged between the two commits. The *only* relevant
diff is `build_swarm.rs` switching to a persisted, shared identity.

### The bug, precisely, and why it wasn't always reproducible

`load_or_create_keypair` (`src/p2p/load_or_create_keypair.rs`) is
read-check-then-write, not locked or atomic:

```rust
if let Ok(bytes) = std::fs::read(&path) {
    if let Ok(keypair) = Keypair::from_protobuf_encoding(&bytes) { return keypair; }
}
let keypair = Keypair::generate_ed25519();
// ... write it out ...
```

- **Sequential launches** (spawn instance A, wait for it to fully start —
  giving it time to generate *and write* `identity.bin` — then spawn
  instance B): B reads A's already-written file and gets A's exact
  identity. **Deterministic collision**, confirmed live via the deskctrl
  MCP (see prior conversation): two windows, each showing only its own
  box, roster stuck at 1 entry, and libp2p log spam of the form
  `Failed NAT traversal ... peer_addr=<the other instance's own address>`
  — i.e. an instance dialing what is, from libp2p's point of view, itself.
- **Near-simultaneous launches** with no identity file yet on disk: both
  processes can reach the `fs::read` check before either has written,
  so both independently generate distinct random keypairs and keep using
  their own in-memory copy for the rest of the run regardless of which one
  "wins" the disk write afterward. **Reproduced directly**: a throwaway
  two-thread harness calling `load_or_create_keypair` against the same
  fresh directory with a `Barrier` (no artificial delay) produced two
  different `PeerId`s in 5/5 runs.

This explains the apparent contradiction between the two manual tests:
- My MCP test explicitly gated the second spawn on the first's
  `wait_for_output` ("Finished") — giving instance A time to write its
  identity file first — so it hit the deterministic-collision path and
  the second window never picked up the first's box.
- Your 13:11 manual test (`/tmp/frt-build/debug/bevy_freenet --p2p-port
  64000` / `64001`, screenshots showing 3 then 2 distinct-colored boxes,
  movement observed syncing both ways) launched both close together via
  the telegram-bot spawn path, with no such gate — consistent with the
  race producing two (or, across repeated runs that session, more)
  distinct identities. Note the build path there (`/tmp/frt-build/...`)
  differs from the one used in this session's MCP tests
  (`~/.cache/frt-build/...`, per `.cargo/config.toml`'s `target-dir`),
  so it's also possible that spawn path runs under a different `$HOME`
  entirely — in which case there would have been no shared file, and no
  race, at all. I could not fully pin down which of these applied to
  your specific run without reproducing your exact spawn environment.

### The fix

`--identity-dir <path>` CLI flag (`src/cli/cli_parse_identity_dir.rs`),
threaded through `p2p::load_or_create_keypair(Option<PathBuf>)` from
`src/main.rs`. Default behavior (no flag) is unchanged — persistent
identity at the `$HOME` path, correct for the normal one-player-per-machine
case. Passing distinct `--identity-dir` values per local instance restores
the pre-`b8128a0` guarantee of distinct identities, deterministically
instead of by race. Verified live: two instances with distinct
`--identity-dir` values converged to a 2-entry roster and showed each
other's boxes.

This does **not** fix the underlying read-check-then-write race in
`load_or_create_keypair` itself — two instances given the *same*
`--identity-dir` (or both hitting the default path with no gate between
launches) can still race. That race is now merely opt-in rather than the
default failure mode. Making the file write atomic (e.g. write-to-temp +
rename, or an advisory lock) would close it fully but hasn't been done.

## Cause 2 (open, upstream, not caused by this project): Freenet update-propagation edge case

Even with distinct identities guaranteed, a controlled test (two instances,
distinct `--identity-dir`, no key input, ~3.5 minutes of wall time) showed
**one-directional** convergence: instance B saw both boxes; instance A never
saw B's, with no errors — just repeated
`freenet::operations::connect: at terminus, cannot accept, uphill budget or
TTL exhausted` and `BROADCAST_NO_TARGETS: scheduling retry` in the logs.

This is not new to our fix — `start_embedded_node.rs` (the code path
responsible for this) is unchanged since the M3 checkpoint that claimed
reliable sync. It's a characteristic of relying on the public Freenet
mainnet's default gateway bootstrap for node discovery (see
`TODO.md`'s "Bootstrapping the relay pool in practice" open question),
made worse by the fact that Freenet itself has an open issue in this exact
area:

> "There may still be an edge case affecting update propagation through
> intermediary peers" — and separately, UPDATE was recently changed to
> fire-and-forget semantics (`freenet-core` PR #2038) but "hasn't been
> tested end-to-end" for multi-hop propagation; "unit tests verify basic
> functionality, but don't catch real-world issues like multi-hop UPDATE
> propagation through actual network." (See freenet.org news and
> `freenet/freenet-core` issue #2045.)

This crate depends on `freenet = "0.2"` / `freenet-stdlib = "0.8"`. This
matches, symptom-for-symptom, what we observed: an UPDATE (our roster
merge) that reaches some peers but not others, with no error, purely
timing-dependent on the DHT path taken. **We did not confirm this is the
literal same code path** (that would need reading `freenet-core`'s source
at the pinned version), but it's the best available explanation for
asymmetric, error-free, retry-looping non-convergence.

### Why our test suite didn't catch either of these

Every existing `testing/tests/two_node_*.rs` test builds its node pair via
`testing::TestNode::start_gateway`/`start_peer`
(`testing/src/methods/test_node/start_node_at.rs`), which sets
`skip_load_from_network: true` and points the peer explicitly at the
gateway's address — a hermetic, directly-wired 2-node network. That's
nothing like what `main.rs` does (default public-mainnet gateway list, no
direct local wiring), and none of those tests exercise
`p2p::load_or_create_keypair` or `p2p::run` at all. A new test,
`testing/tests/e2e_three_node_production_sync.rs`, was added specifically
to exercise the real production path; it is currently red for exactly
Cause 2 (roster convergence across 3 production-path nodes times out at
60s) — see that file's module doc comment.

## Status

| Cause | Status | Where |
|---|---|---|
| 1. Shared/racy libp2p identity | **Fixed** (commit `2ce6f3b`) | `src/p2p/load_or_create_keypair.rs`, `src/cli/cli_parse_identity_dir.rs` |
| 1b. Underlying read-check-write race | **Not fixed** — now pinned by a red diagnostic test | `src/p2p/load_or_create_keypair.rs` (test `concurrent_reads_see_a_stable_on_disk_identity`) |
| 2. Freenet mainnet UPDATE propagation asymmetry | **Bootstrap-only, proven** — direct-wired local path converges; mainnet node-discovery remains unreliable | `src/roster/start_embedded_node.rs` (+ `--freenet-local`/`--freenet-gateway`, hermetic test `testing/tests/local_two_node_production_sync.rs`); mainnet path still tracked by red test `testing/tests/e2e_three_node_production_sync.rs` |

## Update (2026-08-11): diagnosis confirmed, local wiring added

Diagnostics (logging + tests) plus a minimal local-gateway wiring path were added to confirm the two
causes above and give same-machine play a deterministic bypass for Cause 2.

- **Cause 2 is a bootstrap/discovery problem, not a bug in our roster code.** A new hermetic test,
  `testing/tests/local_two_node_production_sync.rs`, runs the exact production startup path
  (`p2p::load_or_create_keypair` → `p2p::run` → `roster::start_embedded_node` →
  `roster::connect_client_loop` → Bevy wiring) for two instances, but wires them directly: one runs
  as an isolated gateway and the other dials it by `"ip:port,hex-pubkey"`. It converges (both
  rosters reach 2 entries, both spawn 2 boxes, movement syncs via libp2p) in ~5s, with no internet.
  Since the only difference from the failing mainnet path is how the nodes discover each other, our
  roster contract/merge/subscribe logic is verified correct and the residual flakiness is purely the
  mainnet node-discovery route.
- **New flags:** `--freenet-local` (isolated gateway host: `skip_load_from_network: true`,
  `is_gateway: true`) and `--freenet-gateway "127.0.0.1:<port>,<hex-pubkey>"` (peer dialing that
  gateway). Both bypass mainnet; neither flag keeps the current default mainnet join. The host logs
  its own dial string so a sibling instance can be launched against it. Wired via
  `cli::parse_freenet_local` / `cli::parse_freenet_gateway` → `connect_and_run` →
  `start_embedded_node(p2p_port, local, gateway)`, which now returns a `roster::NodeInfo` struct
  (`host`, `ws_port`, `public_port`, `public_key_hex`, `node_dir`).
- **Why the production path needs the wiring:** with `skip_load_from_network: false` the node
  fetches the gateway index from `https://freenet.org/keys/gateways.toml` on every boot (fresh temp
  config dir → no cache), sets `relay_ready_connections = Some(3)`, and `wait_ready(0)` proceeds
  with zero connected peers. Two same-machine instances therefore only ever find each other through
  the mainnet DHT routing a freshly-Put contract between them — exactly the fragile, timing-dependent
  path that yields `BROADCAST_NO_TARGETS` / asymmetric convergence.
- **Cause 1b is now reproducible on demand.** The diagnostic test
  `concurrent_reads_see_a_stable_on_disk_identity` (in `src/p2p/load_or_create_keypair.rs`) writes
  the same identity file repeatedly from one thread while readers call `load_or_create_keypair` on
  the same dir. Against the current read-check-then-write `fs::write`, readers catch the
  truncated/partial file and regenerate distinct identities — the on-disk identity flips mid-run and
  the assertion fails. This test is currently RED (that is its point); it goes green once the write
  is made atomic (write-to-temp + rename), which is the remaining Cause 1b fix.
- **Logging added across the production path** so live runs show exactly what each node is doing:
  identity loaded-vs-generated (+ peer id) in `load_or_create_keypair`; join mode, public port and
  pubkey hex in `start_embedded_node`; Get→Found/NotFound and each Update/Put in `setup_contract`;
  and every received roster `UpdateNotification` (with entry count) in `connect_client_loop`.

## Possible next steps for Cause 2

- Give `start_embedded_node` an optional explicit local-gateway wiring path
  (mirroring `testing`'s `start_node_at`'s `gateway` parameter) for when a
  sibling local instance is already known — bypasses the public-mainnet
  propagation question entirely for same-machine/LAN play, which is most of
  what this example needs. **DONE** (2026-08-11): `--freenet-local` /
  `--freenet-gateway` flags wired through `start_embedded_node`; proven by the
  hermetic `local_two_node_production_sync.rs` test.
- Track `freenet/freenet-core` releases past `0.2`/`0.8` for the
  propagation-edge-case fix referenced above, and re-run
  `e2e_three_node_production_sync` after bumping.
- Fix the Cause 1b race properly (atomic write) regardless, since it's a
  correctness bug independent of Cause 2.
