# freenet_libp2p_example — Freenet + libp2p Ping

Prove that freenet and libp2p can coexist in the same process on the same
tokio runtime.

## Goal

A single binary that:
1. Boots a freenet node (persistent layer)
2. Boots a libp2p swarm (real-time layer)
3. Sends/receives libp2p ping messages
4. Prints events from both stacks

## Architecture

```
Binary
  ├── freenet node ─── DHT ──► network
  └── libp2p swarm ─── ping ──► direct peers
```
