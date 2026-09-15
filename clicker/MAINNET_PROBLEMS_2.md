# MAINNET PROBLEMS 2 — GOSSIP + KADEMLIA + RELAY CAMPAIGN

Follow-up to `MAINNET_PROBLEMS.md`. Six instrumented mainnet runs
(3 same-host instances, real public mainnet) after implementing
Freenet-bootstrapped libp2p discovery (gossip roster mirror, kad seeding,
relay/DCUTR wiring, periodic sync, score tombstones).

Result: **0/6 full passes.** Resolution gate now passes everywhere;
2-of-3 exact agreement reached twice. The first-started instance is
systematically starved every run. Everything below is from the logs
(`.local-run/clicker-<timestamp>/instance-{1,2,3}.log`).

## What was built (per plan, all merged, all gates green)

1. **Basis + relay/DCUTR** (`freenet_libp2p_bevy_plugin/src/p2p/`):
   `relay::Behaviour` (server) + `relay::client::Behaviour` + `dcutr`
   wired into the swarm (`build_swarm.rs`, `behaviour.rs`);
   `Command::Dial` honors `peer_id` via `DialOpts` (was ignored),
   `DialForce` (`PeerCondition::Always`), `ReserveRelay`, `AddKadPeer`
   (kad `add_address` + `bootstrap` on first sight);
   `Event::{DialFailed, RelayReserved, ObservedAddr, LobbyProviders}`,
   `Command::{ProvideLobby, FindLobby}` + `lobby_key` provider records.
2. **Gossip roster mirror** (clicker): discovery publishes full slots on
   `clicker/{lobby}/roster` every 5s (`discovery/run.rs:publish_slots`);
   `absorb_roster` merges + dials fresh non-stale entries;
   `Click` moved from unicast `Send` to gossip `Publish`
   (`clicker/click.rs`), new `absorb_click_gossip` sharing
   `credit_click` with `apply_delta`; bridge gained the missing re-put
   leg (`bridge_tick.rs:attempt_reput`); stale entries (>300s) never dial.
3. **Score durability**: kad `Snapshot` chunk stays the shared file;
   unknown snapshot ids park as `ScoreTombstones` (logical key);
   `despawn_on_leave` tombstones labeled leavers;
   `resolve_player` restores tombstoned scores on labeling;
   `request_snapshot` on Startup covers process restarts (`raise_own`).
4. **Mesh TDD harness**: partition mask + connected-component gossip
   routing + `FakeDht` answering `FetchHistory` + turmoil star rig with
   gossip-forwarding pump and per-publish envelope ids. 6 new tests,
   all RED before, all GREEN after (35 mesh tests total green).
5. **Fixes found by mainnet logs between runs**:
   - resolve poisoning → `resolve_player` filters pos topic only.
   - sync-vs-resolve race → shared `label_slot` (color/spot/log) used by
     both `resolve_player` and `drain_pending`.
   - label flapping (41 relabels/run) → `find_slot` never relabels numbered slots.
   - drop-without-redial → `PeerConnected/Direct` link tracking +
     `DialForce` redial of known-but-disconnected peers.
   - simultaneous-dial storm → deterministic dial tie-break
     (lower peer id dials; higher waits for inbound, desperation dial
     after 2 redial periods).

## Run log (all FAIL the converge gate)

| Run | Dir suffix | Time | Shape |
|-----|------------|------|-------|
| 1 | 181335 | 776s | 1 remote max, poisoning `player=4294967296`, ~3 dials total |
| 2 | 182629 | 503s | resolution correct, star persists, connect-storm seen |
| 3 | 183726 | 500s | transitive sync works: 2/3 converge to 87, instance-1 at 51 |
| 4 | 184929 | 231s | label_slot live, flapping found (41 relabels on inst-1) |
| 5 | 190211 | 260s | 2/3 exact agreement (60/60), inst-1 partial (45), force-dial never fires |
| 6 | 191105 | 529s | same structure (39 / 54 / 62), tie-break never triggered |

## Finding 1 — First-starter Freenet singleton never merges (Freenet level)

Every run, instance-1 (first to `Put` the fresh key) ends with a
singleton roster (0 dials all run) while late joiners pull `{1}`,
`{1,2}` via `Get`. Its bridge (subscribe + re-put every 30s, 14–16
firings/run) never heals the split within 4–13 minutes. The app has no
further lever: the node serves its own replica locally and no
notification ever arrives. This is `MAINNET_PROBLEMS.md` Problem 1,
unmoved by the client-side bridge. Conclusion: cold-start merge needs
either much longer runs (5-min anti-entropy may need several cycles),
a node-level fix, or avoiding fresh keys per run.

## Finding 2 — Simultaneous-dial storm kills same-host links (libp2p level)

When two instances dial each other at the same time with multi-addr
`DialOpts` (TCP+QUIC × interfaces + loopback ≈ 8 addrs each way),
logs show connect/disconnect flapping within one millisecond
(C C D C D C D) and the link NEVER establishes afterwards
(runs 2, 5, 6 on the 1↔2 pair). Staggered dials (instance-3 dialing up
seconds later) always succeed and hold for the whole run. The tie-break
(run 6) never got to prove itself: it triggers on learning the peer,
and instance-1 never learns instance-2 (Finding 1 blocks the trigger).

## Finding 3 — Gossipsub direct delivery works; star relay is unproven

3→1 click/pos flow continuously once connected (instance-1 tracks
instance-3's counter 0→16 live). The harness star/turmoil tests prove
center-relayed gossip converges. On mainnet the V topology (1-3-2)
should therefore converge instance-1 via 3 — but instance-1's counts
of instance-2 stay 0. Either 3 doesn't forward into 1 (mesh gap) or
1's churned placeholder slots eat the credits (despawn race). Not yet
distinguished; needs per-topic mesh logging at the swarm layer.

## Finding 4 — Transitive periodic sync works (verified)

5s `SyncReq` heartbeats with full-entry `SyncAck` close the star for
connected pairs: runs 3 and 5 show 2-of-3 exact agreement
(87/87, 60/60). Unicast needs only direct edges; third-party entries
in acks give transitive closure within diameter rounds.

## Finding 5 — Score durability never got a mainnet workout

Tombstone save/restore + snapshot absorb are mesh-green, but on mainnet
no leaver/rejoiner scenario with a working link occurred (links either
hold with no leaves, or never form). Untested live.

## What reliably works now (keep)

- Resolution gate (≥2 `cursor resolved peer=` per log): passes every
  instance in runs 4–6 (was 0–1 before). No poisoning, no flapping.
- 2-of-3 exact agreement whenever the mesh has 2 stable edges.
- Observability: `p2p connected/disconnected`, `p2p dial failed`
  (with reason), `force-dialing`, bridge cadence — every diagnosis in
  runs 2–6 came from these lines.
- Full local gates: clicker 164 + plugin 33 tests, clippy/fmt/lint/
  taxonomy/bevy-lint clean in both crates.

## Recommended next steps (ordered)

1. **Prove the tie-break**: force a simultaneous join (all three spawn
   the same second on fresh keys). If 1↔2 holds, the campaign's core
   thesis is confirmed end-to-end.
2. **Fix or bypass the singleton**: try one 20-minute run (several
   anti-entropy cycles); in parallel ask whether the node exposes any
   stronger merge trigger than subscribe/re-put for a stale replica.
3. **Per-topic gossipsub mesh logging** (swarm layer, debug-gated) to
   settle Finding 3: does the center forward leaf traffic or not?
4. **Single-dialer permanently + smaller addr sets** (prefer loopback/
   LAN TCP, drop QUIC dupes on same-host) to shrink the storm surface
   if (1) still flaps.
5. **Relay rendezvous**: elect the first stable peer as relay and route
   the third pair through a circuit when direct keeps dying.
6. Keep the exact-agreement bar. Do not relax the gate to hide (1).
