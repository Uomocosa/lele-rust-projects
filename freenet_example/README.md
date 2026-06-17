# Freenet Clicker Example

A shared counter that runs across the Freenet P2P network.
Just download and run — no install steps, no dependencies.

## Quick Start

Download the latest binary for your OS:
https://github.com/Uomocosa/lele-rust-projects/releases

```bash
chmod +x freenet-example-linux
./freenet-example-linux
```

The binary starts its own Freenet node, joins the global P2P network,
and increments a shared counter every second. Press Ctrl+C to stop.

Re-run any time to rejoin the same global counter — state lives on
the network, not on your machine.

## Two Machines (Same Counter)

Run the same binary on two machines. Both connect to the global Freenet
network and share the same deterministic contract. No configuration,
no IP sharing, no server.

```bash
# Machine A
./freenet-example-linux

# Machine B
./freenet-example-linux
```

Both increment the same counter. Each sees the other's updates via
pub/sub notifications.

## Development Build

```bash
# Build the WASM contract
cargo build --release --target wasm32-unknown-unknown --manifest-path contract/Cargo.toml

# Build the binary
cargo build --release

# Copy the WASM so the binary can embed it
cp contract/target/wasm32-unknown-unknown/release/clicker_contract.wasm contract/

# Run
cargo run --release
```

## Advanced: External Freenet Node

If you already have a Freenet node running, connect to it with:

```bash
# Deploy and increment (also subscribes)
./freenet-example --role publish

# Subscribe to existing contract
./freenet-example --role subscribe
```

Configure host/port via `FREENET_HOST` and `FREENET_PORT` env vars
(default: `127.0.0.1:7509`).
