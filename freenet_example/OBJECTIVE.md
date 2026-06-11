# freenet_example — Pure Freenet Clicker Game

The simplest possible freenet "app" to prove contract integration works.

## Goal

A shared counter contract where any participant can increment the count and all
subscribers see updates in real time via pub/sub. Demonstrates:

- Deploying a contract to a freenet node
- Subscribing to contract state updates
- Sending updates (increment)
- **Multi-peer: multiple clients incrementing the same counter across machines**

## Architecture

```
Publisher                        Subscriber
    │                                │
    ├─127.0.0.1:7509                 ├─127.0.0.1:7509
    ▼                                ▼
 Local freenet node ─── P2P ───► Local freenet node
       (or same node for single-machine demo)
```

Both clients connect to their **own** local node at `127.0.0.1:7509`. The
deterministic `ContractKey` (hash of WASM + params) is the global address —
the P2P network handles routing. No IP sharing needed.

## Status

| Component | Status |
|-----------|--------|
| Contract (`contract/src/lib.rs`) | ✅ Complete |
| Client WebSocket (`src/connect.rs`) | ✅ Complete |
| Client logic (`src/main.rs`) | ✅ Complete |
| Build automation (`Makefile.toml`) | ✅ Complete |
| Contract unit tests (`contract/src/lib.rs`) | ✅ 2 tests |
| Integration test (`tests/clicker_integration.rs`) | ✅ 1 test |
| Multi-peer (publisher + subscriber) | ✅ Complete (0.0.2) |

## Prerequisites

```bash
# Install cargo-make
cargo install cargo-make

# Install freenet node
git clone https://github.com/freenet/freenet-core.git
cd freenet-core && cargo install --path crates/core

# Ensure wasm target
rustup target add wasm32-unknown-unknown
```

## Usage

### Single client (one terminal)

```bash
cargo make run
```

This will:
1. Build the clicker contract to WASM
2. Start a freenet node in network mode (`--skip-load-from-network`) in the background
3. Wait for the node to be ready
4. Run the client in **publish** mode — deploys the contract, subscribes,
   and increments every second

### Two clients, one machine (pub/sub live updates)

```bash
# Terminal 1: start the node
freenet network --is-gateway --skip-load-from-network

# Terminal 2: wait for node, then run publisher
cargo make run-publisher

# Terminal 3: wait for node, then run subscriber
cargo make run-subscriber
```

Both clients increment the same counter and see each other's updates via
`UpdateNotification` (pub/sub). The counter climbs ~2x per second.

> **Note:** `freenet local` mode does not dispatch `UpdateNotification` to
> subscribers — use `freenet network --is-gateway --skip-load-from-network` for single-machine
> multi-client demos.

### Two clients, two machines

```bash
# Machine A: start a network-mode node, then publisher
freenet
cargo make run-publisher

# Machine B: start a network-mode node, then subscriber
freenet
cargo make run-subscriber
```

No configuration needed — both connect to `127.0.0.1:7509`. The P2P network
routes by the deterministic `ContractKey`. Start the publisher first so the
subscriber's initial `Get` finds the contract.

## How the Subscriber Works

1. Loads the same WASM, computes `ContractKey::from_params_and_code`
2. Sends `Get { subscribe: true, blocking_subscribe: true }`
3. If the publisher hasn't deployed yet, receives `NotFound` and retries
   every second
4. Once found, enters the same increment loop as the publisher
5. Both clients see each other's `UpdateNotification`s via pub/sub

The `ContractKey` is deterministic — `blake3(blake3(wasm) || params)` — so
any client with the same WASM and empty params derives the exact same key
without any network round-trip.

## Configuration

- `FREENET_HOST` env var (default: `127.0.0.1`)
- `FREENET_PORT` env var (default: `7509`)
- `--role publish|subscribe` CLI flag (default: `publish`)
