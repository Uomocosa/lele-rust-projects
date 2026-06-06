# freenet_libp2p_bevy_example — Hybrid Box Game

A two-player box game using the full hybrid networking stack.

## Goal

Prove the complete freenet + libp2p + Bevy integration:

- **Freenet** handles: identity, lobby/discovery, persistent state
- **libp2p** handles: real-time position sync, input events

## Architecture

```
Bevy App
  ├── freenet node ─── DHT ──► lobby, contracts
  └── libp2p swarm ─── direct TCP ──► position sync, input
```

## Current Status

Scaffolded. Plugin, systems, events, and resources exist. Next: implement
the hybrid boxes example logic.
