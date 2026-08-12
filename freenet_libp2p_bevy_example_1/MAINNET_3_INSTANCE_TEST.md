# Three-instance mainnet discovery test — 2026-08-12

Run of three identical `bevy_freenet` release binaries, each with its own
`--identity-dir`, **no** `--freenet-local` and **no** `--freenet-gateway`, i.e.
the pure public-Freenet path that simulates three PCs on three networks.

Contract WASM rebuilt first (`cargo make copy-wasm`); the resulting bytes were
identical to the working-tree copy (`7319cabb…`), so the ContractKey was stable
and this was **not** a fresh-key bootstrap.

| Instance | identity dir | freenet port | result |
|---|---|---|---|
| 1 | `/tmp/fbx_id1` | 56065 | joined, Put the contract, sees 2 entries |
| 2 | `/tmp/fbx_id2` | 38068 | **never joined the roster** |
| 3 | `/tmp/fbx_id3` | 60663 | joined, `merged_len=2`, sees 2 entries |

## Result: discovery works, 2 of 3

Instances 1 and 3 found each other through the public Freenet roster contract
with no local wiring, dialed each other over libp2p, and synced **bidirectionally
in real time**: holding `D` in instance 1 moved its box in instance 3's window,
and holding `A` in instance 3 moved its box in instance 1's window. Both windows
rendered identical box positions throughout.

Instance 2 rendered only its own local box for the whole run.

## Findings

### 1. A failed contract setup is permanent — no retry (highest impact)

`roster::setup_contract`'s initial `Get` (with `blocking_subscribe: true`) hit
the 60 s `recv_timeout` on instance 2:

    ERROR roster: freenet connection error reason="setup failed: timeout after 60s"

`connect_client_loop` then sends `Event::ConnectionError` and **`return`s**
(`src/roster/connect_client_loop.rs:38-43`). Nothing ever retries, so the
instance is dead to the network for the rest of the process lifetime even
though its Freenet node stayed healthy and connected. Note the contrast with
`setup_contract`'s own websocket connect, which *does* retry in a loop — only
the contract op gives up.

Compounding it: the window looks like a perfectly normal single-player session.
There is no on-screen indication that networking died, so the failure is
invisible to the player. M5's "connection-status UI" is the missing piece.

**Suggested fix:** wrap the `setup_contract` call in a retry loop with backoff
rather than returning, and surface `ConnectionError` in the UI.

### 2. Every game client advertises itself as a public Freenet gateway

`src/roster/start_embedded_node.rs`: `let is_gateway = gateway.is_none();` — so
on the mainnet path (`gateway == None`) every instance runs with
`is_gateway: true`. The logs confirm these nodes relay traffic for unrelated
contracts and do NAT traversal on behalf of third-party peers:

    NEIGHBOR_HOSTING: Updated neighbor hosting state peer=5M7v2NZbv6WcLNT5S total_contracts=863
    connect: acceptor accepted joiner, initiating hole punch joiner_addr=77.109.112.13:62043
    connect: at terminus, cannot accept, uphill budget or TTL exhausted — rejecting   (x many)

This was already flagged in `TODO.md` M0.5 ("the embedded `is_gateway: true`
node joins the public Freenet network and relays subscribe traffic for
unrelated contracts. It is not isolated"), but it is still wired that way. A
game client should join as a peer, not serve as network infrastructure — it is
also a plausible contributor to instance 2's 60 s timeout, since the node was
busy servicing other people's connect operations during its own startup.

### 3. `public_address` is hardcoded to loopback on the mainnet path

Same file: `public_address: Some(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)))` is
set unconditionally, including when `local == false && gateway == None`. That
is correct for the same-machine modes but wrong for the stated target of
"2+ machines on different networks" — a real remote peer told to dial
127.0.0.1 would dial itself. Worth verifying whether freenet prefers the
observed address before this is relied on cross-machine.

### 4. No world bounds — boxes leave the playfield

Holding a direction long enough walks a box off the visible area and it stays
there (visible in the run: magenta pinned at the left edge, yellow at the
right). There are no walls and no camera follow, so a player can lose their own
box off-screen with no way to tell where it is.

### 5. Unfiltered log output

`main.rs` builds `tracing_subscriber::fmt()` with no `EnvFilter`, so the whole
freenet crate logs at INFO to stdout — ~300 KB in a few seconds, which buries
the `roster`/`p2p` lines that actually matter. A default of
`RUST_LOG=warn,roster=info,p2p=info` would make the app debuggable.

## Not reproduced / working as intended

- Roster commutative merge, contract Put/Get, libp2p dial-from-roster, 30 Hz
  snapshot sync, and interpolation all behaved correctly for the pair that
  connected.
- Identity persistence per `--identity-dir` worked: three distinct PlayerIds,
  colors, and spawn positions.
- Automatic free-port selection worked (three distinct freenet ports without
  passing `--p2p-port`).
