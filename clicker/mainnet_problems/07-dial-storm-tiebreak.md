# 07. Simultaneous-dial storm and tie-break — MITIGATED

Follow-up to: `MAINNET_PROBLEMS_2.md` Finding 2, `MAINNET_PROBLEMS_3.md` §3.2, `MAINNET_PROBLEMS_4.md` iter-3.
Related: `mainnet_problems/01-cold-start-partition.md`.

> Wording note (`MAINNET_PROBLEMS_10.md`): old files say "Freenet
> dials / hairpin blocks Freenet" — read **libp2p dials seeded by
> Freenet**. All dials/connections are libp2p.

## Symptom

Same-host links flap connect/disconnect within one millisecond
(C C D C D C D) and NEVER establish afterwards; staggered dials
always succeed and hold. Separately: a 5s roster heartbeat re-dialed
every peer every heartbeat (QUIC opens a new sub-connection per dial).

## Evidence (logs)

- `_2` runs 2, 5, 6 on the 1↔2 pair: connect/disconnect flapping
  within 1ms, link never establishes (`.local-run/clicker-<ts>/`,
  dirs deleted — shape only).
- `_3` §2 runs: instance-1 logs show 20–44 connects / 7–17 drops per
  run, often the same peer flapping within one second.
- `_4` iter-3: instance force-dialed an 11×-connected peer every
  ~19s — 5s roster heartbeat bumped `updated_at`, defeating the
  dialed-map.
- Turmoil repros: `hairpin_blocks_same_public_ip.rs` (`Nat::NoHairpin`,
  zero connections — the `_1` P1 / `_2` F2 case, deterministic);
  `cold_start_stagger_converges.rs` (real dials converge).

## Root cause

Two instances dialing each other simultaneously with multi-addr
`DialOpts` (TCP+QUIC × interfaces + loopback ≈ 8 addrs each way).
Plus: heartbeat `updated_at` bumps defeating dial-once tracking.

## Fix (with src:line)

- Deterministic dial tie-break: the lower peer id dials, the higher
  waits for inbound (desperation dial after 2 silent redial periods;
  later halved 60s→15s). Consumers read it via the discovery task's
  15s `redial_missing` (connected-aware).
- `_4` iter-3: dial-once per gossip id in `absorb_roster`; retries
  belong to `redial_missing`, not the heartbeat path. Fast test
  `absorb_roster::connected_peer_never_redialed` (was RED).
- Rig: `plan_for.rs` higher-id-dials tie-break + `redial_missing.rs`
  model; `dial_with_deadline.rs` bounds dials (`Err("timeout")` →
  `p2p::Event::DialFailed`).

## Verification

- Fast: tie-break + never-redialed tests green; turmoil dial cases
  17/17 green (`MAINNET_PROBLEMS_8.md` §4).
- Slow: tie-break never got a live proof while Finding 1 blocked its
  trigger (`_2` F2); staggered dials hold whole runs.

## Open / next slice

- Prove the tie-break with a forced simultaneous join (all three
  spawn the same second on fresh keys).
- Permanent single-dialer + smaller addr sets (prefer loopback/LAN
  TCP, drop QUIC dupes on same-host) if flapping persists.
- Relay rendezvous: elect first stable peer as relay, route third
  pair through a circuit when direct keeps dying.
