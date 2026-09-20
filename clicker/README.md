# Clicker — discovery

> ## Layer rule (read first)
> - FREENET (seed only): room-name → params; room → `{PlayerNo → libp2p PeerId}`.
>   No sockets, no dials, no liveness. `addrs` fields in
>   `src/discovery/peer_entry.rs` and `src/discovery/directory_entry.rs`
>   are one-time dial hints published by `src/discovery/roster_announce.rs`,
>   not connection state.
> - LIBP2P (transport): every dial, PEX, gossip, `SyncReq`/`SyncAck`,
>   `WantJoin`/`Welcome`, heartbeat. Room converge budget ≤500ms same-host
>   (`drive_roster` in `src/discovery/run.rs`).
> - FREENET RING (`freenet-gateway` skill): node bootstrap plumbing only.
>   Never cite it for app-mesh delays. Load that skill for harness bootstrap
>   questions only; all gameplay/dial/converge diagnosis is libp2p.

How instances find rooms and each other without any address ever being
typed, passed, or configured. Two loops run **in parallel for the whole
lifetime of the app**, sharing one hint store.

## The two loops

| | Freenet loop (seed) | libp2p loop (transport) |
|---|---|---|
| Discovers from nothing? | **Yes** — deterministic contract keys route globally, no IPs needed | No — needs at least one contact |
| Direct dial? | No — contracts only carry data | **Yes** — TCP/QUIC dials, relay circuits, hole-punching |
| What it learns | Seed hints (`PlayerId → PeerHint`), room directory | Peer hints + rooms, asked directly |
| Cadence | 1s poll, 5→30s announce, 30s bridge | 5s gossip heartbeat, per-connect PEX, 30s re-PEX, 15s redial |

Start state is a **black vec**: zero known peers, zero known rooms.
libp2p cannot do anything yet. As soon as Freenet returns the first
peer (or room), libp2p dials it and asks it directly what *it* knows.
Every answer grows the known set, and both loops keep looking forever —
new joins, rejoins, and room creates propagate through whichever
channel hears them first.

## Channels (all active, all the time)

1. **Freenet roster contract** — `PlayerId → {peer_id, addrs, updated_at}`
   per room. `Get + subscribe` first, `Put` only on `NotFound`.
   Announcements are single-entry `Update`s. Split-brain heals via
   routed `Subscribe` + idempotent re-`Put` every 30s of silence.
2. **Freenet directory contract** — room name → contract params. Poll
   every 5s; this is how brand-new rooms are discovered from nothing.
3. **Gossip roster mirror** — full known slots published on
   `clicker/{room}/roster` every 5s. Overheard entries merge by
   last-writer-wins and are dialed immediately.
4. **PEX** (`/blackboard` `PexAsk`/`PexResp` over request_response) —
   on every new connection and every 30s per peer: *"here is what I
   know, what do you know?"* Answers carry peer hints **and** rooms
   (with join params), so one contact bootstraps a whole world-view.
5. **mDNS** (optional, on by default) — LAN-only multicast discovery,
   disabled with `--disable-mdns` for WAN/mainnet test runs.

## Merge rules (why loops never fight)

- One hint store, last-writer-wins per `peer_id` on `updated_at`;
  rooms union on merge. A fact learned anywhere is dialable everywhere.
- Caps: 128 peer hints, 32 PEX rooms; entries older than 300s
  (`STALE_ENTRY_SECS`) are never dialed and get pruned.
- One-shot learns always feed the store, so the 15s redial retries
  gossip/PEX-learned peers exactly like freenet-learned ones.
- Dial storm guard: deterministic tie-break — the lower peer id dials,
  the higher waits for inbound (desperation dial after 2 silent redial
  periods). Simultaneous multi-addr dials collapse same-host links
  within milliseconds; the tie-break makes double-dials impossible.

## Game traffic (for orientation, not discovery)

Clicks and cursor moves ride gossipsub topics per room; scores sync
via unicast `SyncReq`/`SyncAck` heartbeats (transitive: a center peer's
ack carries third-party entries) plus kad history snapshots. None of
this discovers anyone — it only flows once discovery connected them.

## Mainnet problems and solutions

All observed mainnet failures, their fixes, and open slices live in
`mainnet_problems/` (one file per problem), indexed by
`MAINNET_PROBLEMS_INDEX.md` — start there. The index also documents
how to add new problems/solutions.
