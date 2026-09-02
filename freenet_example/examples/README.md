# Examples

This directory contains runnable examples demonstrating different ways to use the Freenet counter contract.

## Local Examples (single machine)

These examples start an in-process Freenet node on your machine and require no external setup.

| Example | Command | What it shows |
|---------|---------|---------------|
| `standalone_demo` | `cargo run --example standalone_demo` | Start a node, deploy the counter, tick it 3x, print state |
| `publish_subscribe` | `cargo run --example publish_subscribe` | Two clients (publisher + subscriber) sharing state through one node |

## Remote Examples (connect to an existing node)

Connects to a Freenet node that is already running (either local or remote).

| Example | Command | What it shows |
|---------|---------|---------------|
| `connect_to_external` | `cargo run --example connect_to_external` | Connect to any remote node's WebSocket API via `FREENET_HOST`/`FREENET_PORT` |

## P2P Examples (automatic discovery across machines)

These examples demonstrate Freenet's peer-to-peer contract synchronization. When multiple people run the same example, they automatically share the same counter — updates from any participant are visible to all others.

### Approach A: Public gateways (fully automatic)

```bash
cargo run --example p2p_counter
```

The node fetches the public gateway index from `https://freenet.org/keys/gateways.toml`
(currently `nova.locut.us:31337` and `vega.locut.us:31337`), joins the global Freenet
network, and discovers peers hosting the same contract via P2P routing. **No
configuration needed** — just run on any machine with internet access. This is the
recommended path matching the zero-infrastructure project goal.

### Approach B: Dedicated gateway (controlled demo)

On the gateway machine:
```bash
cargo run --example p2p_counter_gateway -- --gateway --public-address <YOUR_IP>
```

On each peer machine:
```bash
cargo run --example p2p_counter_gateway -- --connect <GATEWAY_IP>:31337,<GATEWAY_PUBKEY>
```

The gateway machine acts as a bootstrap node. All peers connect to it, forming a private P2P network. Works offline or on LAN. Requires running your own gateway — does not match the zero-infrastructure goal.

### Approach C: WebSocket bridge (single node, multiple clients)

On the host machine:
```bash
cargo run --example p2p_counter_ws_bridge -- --host <YOUR_IP>
```

On each client machine:
```bash
cargo run --example p2p_counter_ws_bridge -- --connect <HOST_IP>:<PORT>
```

All clients connect directly to the same node's WebSocket API. The counter contract lives on that one node. This is the simplest multi-machine demo but has a single point of failure. Requires running your own host — does not match the zero-infrastructure goal.

## How Freenet P2P Works

Each machine runs a Freenet node. Your client app connects to its **local** node via WebSocket at `127.0.0.1:<port>`. The local node then communicates with other nodes via the Freenet P2P network.

Contracts are addressed by a deterministic key derived from the contract's WASM code and parameters. When you subscribe to a contract key, the P2P network routes your subscription to the peer(s) hosting that contract — no IP addresses need to be shared manually.

**Prerequisites for P2P examples:**

- The contract WASM must be built first:
  ```bash
  cd contract && cargo build --release --target wasm32-unknown-unknown && cd ..
  ```
- The binary must be the same on all machines (same WASM code → same contract key)
- For Approach A, internet access to `freenet.org` is required
- Firewall rules may need to allow inbound connections on port `31337` (configurable)

## Verifying Across Two Machines (Approach A, public gateways)

Approach A is the zero-infra path: every participant fetches the live public gateway
index from `https://freenet.org/keys/gateways.toml` (currently `nova.locut.us:31337`
and `vega.locut.us:31337`) and joins the same global Freenet network. Anyone running
the same contract on any OS — Linux, macOS, Windows — participates in the same
shared counter.

### What you'll observe

- **State is global and persistent.** `counter deployed, initial count: N` where `N`
  is whatever the last writer on Earth left behind. Re-run any time — you rejoin the
  same global counter, you don't reset it.
- **Updates are published.** Each `tick k: count = M` proves your local update was
  accepted by the contract state on the network.
- **Cross-peer notifications can lag.** The `p2p_counter` example's `tick` loop only
  drains 10 ms of pub/sub notifications before issuing its own update, so you may
  *not* see other peers' increments reflected in your own `tick` output even though
  they are landing on the contract state. To visibly converge in real time, either:
  1. Lengthen the notification drain window (e.g. change `recv_timeout(Duration::from_millis(10))`
     to `recv_timeout(Duration::from_millis(500))` in `src/GlobalCounterClientMethod/tick.rs`), or
  2. Periodically issue a `get` to refresh local state from the contract.

The contract state itself is authoritative and shared across the planet — only the
*client-side display* can lag.

### Step-by-step: two physical machines (e.g. Linux + macOS)

1. Build the binaries (or grab prebuilt ones from the Releases page when a tag is cut).
   ```bash
   # Linux
   cargo build --release --manifest-path freenet_example/Cargo.toml
   # The example binary lives at freenet_example/target/release/examples/p2p_counter
   ```

2. On machine A (Linux):
   ```bash
   ./freenet_example/target/release/examples/p2p_counter
   ```

3. Within ~5 seconds on machine B (macOS or Windows):
   ```bash
   ./p2p_counter
   ```

4. Both should report the **same starting count** (within a couple of ticks) and
   continue producing their own updates. Global contract state advances; inspecting
   it via `get` from either peer reveals the latest value regardless of which peer
   wrote it.

### Two instances on one machine (Approach A)

You can also run two `p2p_counter` instances on the same machine — each uses a unique
tempdir (`tempfile::tempdir()`), so they don't share local state. From the network's
point of view they are two independent peers.

```bash
# Terminal 1
cargo run --example p2p_counter
# Terminal 2 (within ~5 s)
cargo run --example p2p_counter
```

Both will retrieve the same persisted global count and both will publish updates
to the same contract. This is the fastest way to verify the end-to-end plumbing
(P2P join + state retrieval + update publication) without a second machine.