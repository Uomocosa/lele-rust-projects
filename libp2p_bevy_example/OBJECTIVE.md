# OBJECTIVE: Bevy P2P Library

## Role
You are an expert Rust game networking engineer. Your task is to build a zero-infrastructure, fully decentralized P2P library for the `bevy` engine using `rust-libp2p`.

## Project Phases & Goals

**Phase 1: Native Desktop (Current Focus)**
1. Implement automatic mDNS discovery to find local peers.
2. Implement manual connection dialing via Multiaddrs.
3. Build two example games to test multiplayer groundwork:
   - **Boxes**: Platformer (Left, Right, Jump). Each player controls their own box.
     - On `PlayerJoined`: Spawn a new box for the joining player
     - On `PlayerLeft`: Despawn the disconnected player's box
   - **Clicker**: Click game. Each player has their own button with label "You".
     - On `PlayerJoined`: Spawn a new button for the joining player (label "Opponent")
     - On `PlayerLeft`: Despawn the disconnected player's button
     - Self-click: +1 to your score. Opponent-click: -1 to their score.
     - Labels: "You: N" (your score), "Opponent: N" (opponent score)
4. Create multiplayer groundwork: `Plugin` events and handler system for games.
5. Testing: Simulate discovered/joined players without real P2P (`FakeNetwork` for this phase only).

**Phase 1.5: Multiplayer Plugin Framework**
- Build configurable `Plugin` with builder pattern: `Plugin::new(config)`, `Plugin::coop()`, `Plugin::pvp()`, etc.
- Events games receive via handler system:
  - `DiscoveredPlayer` - New peer found via discovery
  - `JoinRequest` - Peer requests to join (return Accept/Reject)
  - `PlayerJoined` - Peer connected, spawn their entity
  - `PlayerLeft` - Peer disconnected, cleanup
  - `NetworkMessage` - Data received from peer
- Configurable: discovery (mDNS, manual dial), message serialization, heartbeat interval
- Testing: `FakeNetwork` resource to simulate discovered/joined players without real P2P connections (In the next phases we will also add real network tests.)

**Phase 2: Browser (WebAssembly)**
*Constraint Awareness:* Browsers cannot use mDNS. This phase will require WebRTC/WebTransport and a temporary signaling server for the initial handshake. Keep Phase 1's architecture modular enough to swap out the mDNS discovery layer for a WebRTC layer later.

## Execution Rules
- Default to the simplest possible implementation that satisfies the requirement.
- Do not introduce authoritative servers; the architecture must remain peer-to-peer.
- Games implement their own multiplayer logic via handler system; `Plugin` handles discovery, connection, and message routing.
- Sensible defaults (`coop`, `pvp`, `mmo`, `lan_coop`, `lan_pvp`) as starting points that devs can override.
