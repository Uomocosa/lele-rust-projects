# Freenet Clicker Example

A shared counter contract with real-time pub/sub across multiple clients.

## Quick Start

```bash
cargo make run
```

Builds the contract, starts a gateway node, and runs the publisher.

## Two Clients (Same Machine)

```bash
# Terminal 1: start node
freenet network --is-gateway --skip-load-from-network --public-network-address 127.0.0.1 --public-network-port 31337

# Terminal 2: publisher (subscribes + increments; deploys if new)
cargo run -- --role publish

# Terminal 3: subscriber (subscribes + increments)
cargo run -- --role subscribe
```

Both clients show `received update notification` for each other's increments.
Counter converges correctly (~2x per second when both running). The count
persists across restarts — publisher only resets on first deploy.

> **Note:** `freenet local` does not dispatch `UpdateNotification` to subscribers.
> Use network mode for multi-client pub/sub.

## Two Machines

```bash
# Machine A
freenet
cargo run -- --role publish

# Machine B
freenet
cargo run -- --role subscribe
```

No configuration needed — both connect to `127.0.0.1:7509`. The P2P network routes
by the deterministic `ContractKey`. No IP sharing required.

## Build

```bash
cargo make build-contract    # WASM contract
cargo build --release        # client binary
```

## Clear Node State

If the contract was previously marked broken (non-idempotent), clear the DB:

```bash
rm -rf ~/.local/share/freenet/db
```
