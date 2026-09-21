# 15. Hint-union dial-before-Get + synthetic-connect dedup — MITIGATED

Follow-up to: `mainnet_problems/08-roster-get-stall.md` (Q1 structural slice),
`mainnet_problems/12-leave-sticks-dead-cursor.md` (F1 survivor ghost),
`mainnet_problems/10-join-handshake-welcome.md` (reveal path).
Related: `mainnet_problems/07-dial-storm-tiebreak.md`, `mainnet_problems/13-converge-budget-500ms.md`.

## Symptom

Two slow-gate failure modes on `freenet:run-rooms-rejoin`, both downstream of
the same design flaw (roster `Get` gating libp2p dials):

1. **Stall kills the gate (08/F4).** Creator `Get` times out 2–4×
   (`response timed out`, 10s `CONNECT_TIMEOUT_SECS`); `expected` never
   publishes; `create-ready-1` panics at 60s. 2/5 RED in the `_13` campaign.
2. **Leaver never despawns (12/F1).** After app3 is SIGKILLed, survivors keep
   `players=1,2,3` for 80s+ (grace is 15s). Q1 campaign run 1
   (`.local-run/rooms-rejoin-20260921-050904/`).

## Evidence (logs)

- Q1 run 1, `instance-1.log:666,831,1030` — creator `Get` timed out 3×
  (05:09:36, 05:09:51, 05:10:06) yet the gate proceeded: pre-Get union
  (`instance-1.log:298` `expected hint-union published before roster get
  peers=0`) resolved the gate at once, and the `Get` landed later
  (`instance-1.log:1108` `expected set published peers=1`). Same shape on
  `instance-2.log:271,487,614,703` (2 timeouts, then `peers=2`).
- Q1 run 1, main `p2p connected/disconnected peer=12D3KooWCww9YK…`
  (dead app3): 7 connects vs 7 balanced real disconnects in
  `instance-1.log`, yet the labeled slot survived — one extra phantom
  connect stranded the plugin refcount (see Root cause 2).
- Q1 runs 2–5: zero create/join-ready REDs (was 2/5); `leave-owners-*`
  4/4 green after fix 2.

## Root cause

1. `switch_room` / initial connect published `expected` only AFTER the
   blocking roster `Get` (`src/discovery/run.rs`), so a stalled `Get`
   starved the libp2p data plane of dial targets and the Bevy gate of its
   `expected` set. Layer violation in reverse: Freenet slowness blocked
   libp2p progress that needed no contract state.
2. `poll_expected` pushed a synthetic `PeerConnected` for EVERY expected
   publish (`src/lobby/bevy_systems/poll_expected.rs`). Each phantom
   connect is a permanent +1 on the plugin `poll_roster` link refcount
   with no matching disconnect, so one re-publish strands the dead peer
   in `roster::Roster` forever and `despawn_on_leave`'s 15s grace never
   starts. Proven by the fast RED below (`["peer-2", "peer-2"]`).

## Fix (with src:line)

- **Hint union** (`src/discovery/hint_union.rs`, new): pure
  `hint_union(directory, pex, own_peer_id, now_secs)` — directory
  publishers + PEX hints, fresh-only (`STALE_ENTRY_SECS`), own/empty
  excluded, sorted/deduped. Time injected, no clock reads.
- **Dial before Get** (`src/discovery/run.rs`): `publish_pre_get_union`
  sends the union at initial connect; `send_hint_union` sends + dials
  (via `merge_peer_hints` + `dial_hint`, tiebreak-guarded) at the top of
  `switch_room`; `connect_roster_retry` keeps the blocking `Get` in the
  background and `send_expected` union-upgrades on success;
  `dial_directory_publishers` dials directory publishers at connect with
  timestamp continuity into `RunContext.attempted` (no double-dial
  storm). Gossip room-tagging in `drain_lobby_events` stays TODO
  (untagged union dials everyone known; PEX-confirm filters at join).
- **DialFailed prune** (`src/main.rs`, `src/discovery/run_config.rs`,
  `src/discovery/run.rs: `drain_link_events``): `DialFailed` now routes
  into discovery; prune drops stagger queue + pex hint, keeps `attempted`
  (tiebreak/backoff intact), logs `discovery: pruned dial-failed peer`.
- **Synthetic-connect dedup** (`src/lobby/bevy_systems/poll_expected.rs`):
  synthesize `PeerConnected` only for peers not in the previous expected
  set (first publish announces all; re-publishes announce growth only).
  Welcome-known peers need no synthetic (already in the roster via
  `absorb_welcome`).

## Verification

- Fast: `hint_union` 3 tests (union/dedup, own/empty/stale, empty→empty);
  `poll_expected` 2 new tests (`republish_same_set_announces_nothing`,
  `growth_announces_only_newcomers`) — both RED before the fix
  (`["peer-2", "peer-2"]`), GREEN after. Full `lele:nextest` 379/379;
  `clippy -D warnings`, `fmt --check`, `lele_lint`, `lele_bevy_lint`,
  `taxonomy_check` clean.
- Slow: `freenet:run-rooms-rejoin` Q1 campaign (5 runs, Sep-21):
  run 1 RED `leave-owners-1` (pre-dedup) → runs 2–5 with the fix:
  43/44 (budget 539ms), 42/44 (2× reveal race), 42/44 (2× reveal race),
  41/44 (budget 526ms + 2× reveal race). Stall-kill REDs 0/5;
  `leave-owners` 4/4 after the dedup fix.

## Open / next slice

- **Reveal-log race (new).** `rejoin-3`/`rejoin-1` checks assert a
  survivor logs `join reveal`, which fires only on the handshake path;
  when gossip-Move labeling (R1) wins the race the rejoin is fully
  functional (owners 3/3, state intact) but the line never prints. 3/5
  Q1 runs hit it; `_13` hit 0/5 — scatter, not regression. Proposal:
  harden the assertion to accept either reveal-log or gossip-label
  evidence (user call), or force handshake-first ordering.
- **Converge budget (13).** Q1 measurements: 539ms, 242ms, 148ms,
  526ms — still scatters across 500ms. Relax-to-1s stays the user's
  call; `phase_finish` promotion mechanics unchanged.
- **Gossip room-tagging.** `drain_lobby_events` still sets
  `rooms: Vec::new()`; tag gossip/PEX hints by room when the dial
  volume justifies it.
