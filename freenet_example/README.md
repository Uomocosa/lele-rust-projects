# Freenet Clicker Example

A shared counter contract with real-time pub/sub.

## Quick Start

```bash
cargo make run
```

## Two Clients (Same Machine)

```bash
# Terminal 1: start node
freenet network --is-gateway --skip-load-from-network --public-network-address 127.0.0.1 --public-network-port 31337

# Terminal 2: publisher (deploys + increments)
cargo run -- --role publish 2>&1

# Terminal 3: subscriber (subscribes + increments)
cargo run -- --role subscribe 2>&1
```

> **Note:** Run `freenet local` instead for single-client testing. Pub/sub notifications only work in network mode.

## Two Machines

```bash
# Machine A
freenet
cargo run -- --role publish

# Machine B
freenet
cargo run -- --role subscribe
```

## Clear Node State

```bash
rm -rf ~/.local/share/freenet/db
```

## Build

```bash
cargo make build-contract
cargo build --release
```
