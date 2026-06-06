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
Client (main.rs) ── HTTP/WS ──► Local freenet node ── DHT ──► Network
       │                                 ▲
       └───── subscribe & print updates ─┘
```

The contract is a WASM module; the client is a native binary.
