# 01. Cold-start partition — MITIGATED

Follow-up to: `MAINNET_PROBLEMS.md` Problem 1, `MAINNET_PROBLEMS_2.md` Finding 1, `MAINNET_PROBLEMS_3.md` §3.1.
Related: `mainnet_problems/07-dial-storm-tiebreak.md`, `mainnet_problems/08-roster-get-stall.md`.

## Symptom

Three simultaneous first `Put`s of a fresh contract key seed three
disjoint singleton replica groups. Nodes never intersect: no
cross-sibling relays, `BROADCAST_NO_TARGETS`, bridge subscribes route
to non-hosts. Runs split ~50/50 between "mesh forms" and "total
isolation" with no code change in between.

## Evidence (logs)

- `MAINNET_PROBLEMS.md` Problem 1: run 1 (fresh key) — 12 min, zero
  foreign entries anywhere (`Get`/`Subscribe` on `.local-run/` logs of
  that campaign, run dirs since deleted — shape only, no lines).
- `MAINNET_PROBLEMS_2.md` Finding 1: every run, instance-1 (first to
  `Put`) ends with a singleton roster (0 dials all run) while late
  joiners pull `{1}`, `{1,2}` via `Get`. Bridge (subscribe + re-put
  every 30s, 14–16 firings/run) never heals the split within 4–13 min.
- `MAINNET_PROBLEMS_3.md` §2: UI-era runs `rooms-rejoin-083029`
  (instance-2 never pulls room in 180s, directory split),
  `rooms-rejoin-084339` / `085533` (total isolation, p2p 0/0) vs
  `rooms-rejoin-084919` (near miss, full mesh) — same tree, lottery.
- Turmoil repro: `tests/turmoil/connect/cold_start_stagger_converges.rs`
  (`COLD` profile, 1s stagger) converges with real dials.

## Root cause

Freenet-level replica split on concurrent fresh-key `Put`s, unmoved by
the client-side bridge (subscribe + re-put every 30s). Same-public-IP
NAT without hairpin (`Outbound handshake failed: max connection
attempts`) prevents the singletons from meeting at the libp2p layer
either. Anti-entropy (5-min, neighbor-pair-only) never pairs the right
neighbors within run timeouts.

## Fix (mitigations, with src:line)

- Stagger spawns by Freenet readiness — spawn N+1 only after
  `discovery: roster connected` appears in N's log (test harness,
  `tests/e2e_local/mainnet.rs`).
- UI suite: late-Get pull gating (`spawn_puller` respawns on missed
  pull), `directory listed rooms` gate before create/join
  (`MAINNET_PROBLEMS_3.md` §1 Track B).
- Test-only fresh keys: per-run unique `CLICKER_CONTRACT_PARAMS`
  (`MAINNET_PROBLEMS_5.md` §7) so runs stop contaminating each other —
  does not fix the race, isolates it.
- Bridge re-put leg (`src/discovery/bridge_tick.rs:attempt_reput`) —
  narrows but does not close the split.

## Verification

- `MAINNET_PROBLEMS.md`: 2 passes (118s, 254s) with stagger; run 2
  passed only because instance-2 joined ~2 min late and `Get` pulled
  mature state.
- `MAINNET_PROBLEMS_4.md` §4: `rooms_rejoin` PASS (546s),
  `mainnet_local` PASS (103s) on the staggered tree.
- No fast test possible by construction (needs real ring routing).

## Open / next slice

- One 20-minute run (several anti-entropy cycles) to test slow heal.
- Ask whether the node exposes any stronger merge trigger than
  subscribe/re-put for a stale replica.
- Structural: roster `Get` off the click path (`08-roster-get-stall.md`).
