# freenet_libp2p_bevy_example_1 — Hybrid Box Game

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

M0 (scaffolding), M1 (local avian2d physics game, no networking), and M2
(roster contract on Freenet) are done. The game has a `contract/` crate
with a commutative-merge roster (`BTreeMap<PlayerId, PeerEntry>`), an
embedded Freenet node with a real readiness check (no blind sleeps), a
`roster` domain that spawns a box for every roster entry, and a `testing/`
crate whose `two_node_roster` test proves two separate embedded nodes
actually join and converge on the same 2-entry roster. See `TODO.md` for
the full milestone plan and `M2_STEP.md` for the M2 design writeup.

Next: M3 — libp2p real-time position sync (same-machine/LAN first).
