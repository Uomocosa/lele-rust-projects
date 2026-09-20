# 10. Join handshake (WantJoin/Welcome) + quiet-period readiness — SOLVED

Follow-up to: `MAINNET_PROBLEMS_5.md` §§13–14 (commit `733fd77`).
Related: `mainnet_problems/09-instant-join-path.md`.

## Symptom

Strict full-roster gate stalled forever: `Welcome`-union advertises
peers the joiner can never dial/sync, so no `join ready` while game
traffic already flowed (first slow gate RED on the new protocol).

## Evidence (logs)

- Pre-`733fd77` slow gates: 0/5 green (tail reds on churned rings).
- Post-`733fd77` slow gate: PASS (583s) — prior tail reds absorbed
  by the quiet gate.
- TDD incident: `Welcome` re-added a mesh-harness roster entry under
  a second (`blake3`) key, resurrecting a despawned slot and REDDING
  `rejoin::leaver_rejoins_at_old_score` — fixed with value-dedupe on
  insert (same ghost family as `_4` iter-2).

## Fix (with src:line)

- New `CursorMsg::{WantJoin{room}, Welcome{room, peers, own_score,
  joining}}` (`src/clicker/cursor_msg.rs:32-41`) + three Bevy systems:
  `send_want_join.rs` (idempotent per-peer ask, event restored),
  `answer_join.rs` (roster names + own authoritative score; wrong-room
  dropped), `absorb_welcome.rs` (unions sender+peers into
  roster/`JoinGate.expected`; score becomes a synthetic `SyncAck` so
  the existing `absorb_sync` merge path does the work).
- Spectate: `spawn_on_join.rs` no longer gates on `JoinPending` —
  remotes render while loading, own clicks stay parked.
- Quiet-period readiness: `JoinClock{clicked_at, last_new_peer}`
  (`src/lobby/join_clock.rs`) + `QUIET_SECS=1` / `JOIN_CAP_SECS=30`
  (`src/lobby/constants.rs:3-4`); `clear_pending.rs` clears on
  1s-silence-since-last-*new*-peer or 30s cap; growth-only bumps
  (duplicates never reset quiet). Contract wasm unchanged.

## Verification

- Fast: 273 nextest green; `tests/mesh/join.rs` (handshake,
  roster+gate union, spectate), `clears_ghost_once_quiet`,
  `clears_on_alone_cap`.
- Slow: `rooms_rejoin` PASS (583s).

## Open / next slice

- Scores merge eagerly during spectate (not strict positions-only);
  `Welcome.joining` wired but always empty; no reply jitter; roster
  `Get` still on the click path.
- `WantJoin`/`Welcome` over loss (`flaky-nat`) — `join.rs` is
  in-memory only today.
