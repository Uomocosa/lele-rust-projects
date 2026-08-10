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

M0 scaffolding complete: old `src/` removed, `Cargo.toml` pinned to bevy
0.19 / avian2d 0.7 / freenet 0.2 / freenet-stdlib 0.8 / the full libp2p
feature set (quic, relay, dcutr, autonat, etc.), `.cargo/config.toml` and
`Makefile.toml` copied from `freenet_bevy_example_2` (with the
`run-publisher`/`run-subscriber` tasks dropped). `cargo build --all-targets`
is green against a placeholder `src/lib.rs`. `build.rs` (contract WASM
build) is deferred to M2, which is when `contract/` first exists.

See `TODO.md` for the full milestone plan. Next: M1 — local game, no
networking (avian2d physics, keyboard-controlled box).
