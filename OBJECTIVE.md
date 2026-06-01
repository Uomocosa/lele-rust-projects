# OBJECTIVE

Create a hybrid **Freenet + libp2p** networking stack for **Bevy** games, demonstrated through working examples.

## Architecture

```
                ┌─────────────────────────────────────────┐
                │  Game (Bevy App)                         │
                │                                          │
                │  ┌──────────────────┐  libp2p           │
                │  │ Real-time layer  │◄──── direct p2p ──►│ other player
                │  │ (TCP/QUIC)      │  <5ms latency      │
                │  ├──────────────────┤                    │
                │  │ Persistent layer │  freenet           │
                │  │ (contracts)      │◄──── DHT ─────────►│ network
                │  │ identity, lobby, │                    │
                │  │ leaderboards     │                    │
                │  └──────────────────┘                    │
                └─────────────────────────────────────────┘
```

**Freenet handles:** identity, discovery, lobby/matchmaking, persistent state (contracts).
**libp2p handles:** real-time position sync, input events, chat — sub-5ms direct p2p streams.

## Goals

1. **Example: Clicker game** — Pure freenet. Shared counter via contract, pub/sub for updates. Proves the contract integration works.
2. **Example: Two-player Box game** — Hybrid. Freenet for lobby/discovery, libp2p for real-time position sync. Proves the full stack.
3. **Generalize** — Extract reusable patterns from both examples into a `bevy_freenet` plugin + template.

## Current Phase

Hybrid design. libp2p skill created. Next: write the hybrid boxes example.
