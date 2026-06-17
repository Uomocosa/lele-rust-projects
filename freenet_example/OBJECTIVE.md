# freenet_example

A shared counter that anyone can run — just download, execute, and you're
participating in a global shared state over the Freenet P2P network.

## Goal

**A single executable that works on any platform.** Anyone downloads it,
runs it, and their machine immediately becomes part of a shared counter
contract across the Freenet network. No install steps, no toolchain, no
dependencies.

The counter persists on the network — restarting the binary fetches the
current global state, not a local cache.

## How it works

```
Your machine                   Friend's machine
    │                                │
    ├─127.0.0.1:7509                 ├─127.0.0.1:7509
    ▼                                ▼
 Local freenet node ─── P2P ───► Local freenet node
```

Each machine runs its own Freenet node (embedded in the binary). The nodes
sync contract state via the global Freenet P2P network. The deterministic
`ContractKey` (hash of WASM + params) is the global address — no IP sharing,
no server, no configuration.

The subscriber:
1. Loads the same WASM, computes the same deterministic `ContractKey`
2. Sends `Get { subscribe: true }`
3. If the contract doesn't exist yet, retries every second
4. Once found, joins the increment loop alongside the publisher
5. Both see each other's updates via pub/sub notifications

## Achieved

- A single executable that starts an in-process Freenet node, deploys the
  contract, joins the global P2P network, and increments every second
- Cross-platform: CI builds and validates on Linux, macOS, Windows
- No dependencies: WASM embedded at compile time, node runs in-process
- A clicker WASM contract with validate, update, summarize, and delta logic
- A WebSocket client library (`FreenetClient`) for talking to a Freenet node
- A `ClickerClient` that handles the full lifecycle
- Automated tests with in-process network-mode node covering lifecycle,
  pub/sub, persistence, and concurrent writers

## Get it

Download the latest binary for your OS from
https://github.com/Uomocosa/lele-rust-projects/releases

```bash
chmod +x freenet-example-linux
./freenet-example-linux
```

Press Ctrl+C to stop. Re-run to rejoin the global counter.
