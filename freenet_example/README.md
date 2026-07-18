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
# Build and run — build.rs handles the WASM contract automatically
cargo build --release
cargo run --release

# Or with a custom P2P port
cargo run --release -- --p2p-port 41338
```

### Prerequisites

- Rust toolchain with `wasm32-unknown-unknown` target:
  ```bash
  rustup target add wasm32-unknown-unknown
  ```

## Testing

Three test tiers, from fast/offline to slow/online:

| Command | What | Internet? |
|---------|------|:---:|
| `cargo test --all-targets` | Contract (2) + library (12) + integration (8) — 22 tests | No |
| `cargo make e2e` | Binary smoke + two-instance P2P sync + WS bridge | Yes (sync test needs Freenet P2P) |
| `cargo make pre-push` | Build + clippy + fmt + all tests + e2e | Yes |

Individual e2e tests:
```bash
cargo test --manifest-path e2e_tests/Cargo.toml --release --test smoke
cargo test --manifest-path e2e_tests/Cargo.toml --release --test two_instances_sync
cargo test --manifest-path e2e_tests/Cargo.toml --release --test ws_bridge_sync
```

Run with `--nocapture` to see the binary's output.

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
