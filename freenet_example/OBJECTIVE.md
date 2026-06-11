# freenet_example — Pure Freenet Clicker Game

The simplest possible freenet "app" to prove contract integration works.

## Goal

A shared counter contract where any participant can increment the count and all
subscribers see updates in real time via pub/sub. Demonstrates:

- Deploying a contract to a local freenet node
- Subscribing to contract state updates
- Sending updates (increment)

## Architecture

```
Client (main.rs) ── WebSocket ──► Local freenet node (port 7509)
       │                                 ▲
       └───── subscribe & print updates ─┘
```

The contract is a WASM module; the client is a native Rust binary that
communicates via the freenet WebSocket API using the native (bincode) protocol.

## Status

| Component | Status |
|-----------|--------|
| Contract (`contract/src/lib.rs`) | ✅ Complete |
| Client WebSocket (`src/connect.rs`) | ✅ Complete |
| Client logic (`src/main.rs`) | ✅ Complete |
| Build automation (`Makefile.toml`) | ✅ Complete |
| Contract unit tests (`contract/src/lib.rs`) | ✅ 2 tests |
| Integration test (`tests/clicker_integration.rs`) | ✅ 1 test |

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

```bash
cargo make run
```

This will:
1. Build the clicker contract to WASM
2. Start a local freenet node in the background
3. Wait for the node to be ready
4. Run the client, which deploys the contract, subscribes, and increments every second

## Configuration

- `FREENET_HOST` env var (default: `127.0.0.1`)
- `FREENET_PORT` env var (default: `7509`)
