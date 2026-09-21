# 16. Rejoin reveal log flake (`join reveal` missing) — OPEN

Follow-up to: `mainnet_problems/13-converge-budget-500ms.md` verification.
Related: `mainnet_problems/10-join-handshake-welcome.md`, `mainnet_problems/12-leave-sticks-dead-cursor.md`, `mainnet_problems/15-hint-union-dial-before-get.md`, `mainnet_problems/17-menu-double-despawn.md`, `mainnet_problems/18-leave-button-ui-flake.md`.

## Symptom

After the app-3 and creator rejoins, the `rejoin-3` / `rejoin-1`
checks "a survivor logged the shared reveal after the rejoin" fail:
`instance-1.log` / `instance-2.log` carry no `join reveal` line from
`reveal_on_join` (`src/clicker/bevy_systems/reveal_on_join.rs:53`),
even though every agreement/owner check passes.

`phase_finish` promotes the soft ❌ to fatal, so the slow gate REDs on
it alone. The room is functionally intact every time: owners 3/3,
`rejoin-restores` (no state lost), `creator-rejoin-agreement`, and the
500ms converge budget (48–58ms) all pass.

## Evidence (logs)

5× `freenet:run-rooms-rejoin` (Sep-21, 11:22–11:45 local), run dirs
`clicker/.local-run/rooms-rejoin-20260921-*`:

| # | run dir (suffix) | result | secs | failure |
|---|---|---|---|---|
| 1 | `092330` | PASS 44/44 | 291 | — (rejoin-1 is a false pass, see below) |
| 2 | `092745` | FAIL 42/44 | 253 | `rejoin-3` + `rejoin-1` reveal |
| 3 | `093159` | FAIL 42/44 | 253 | `rejoin-3` + `rejoin-1` reveal |
| 4 | `093612` | FAIL (early) | 235 | `leave-owners-2` (3 players, expected 2) |
| 5 | `094021` | PASS 44/44 | 333 | — (rejoin-1 is a false pass, see below) |

**The `rejoin-1` check is a false pass.** `assert_rejoin_visible`
(`tests/e2e_local/rooms_rejoin.rs:190-195`) scans the *whole* survivor
log with `log_contains` (`:40`), not the tail after the rejoin click.
Because `rejoin-3` runs first against the same `instance-2.log`, its
`join reveal` satisfies `rejoin-1` forever:

- `rooms-rejoin-20260921-094021/instance-2.log:1921`
  (09:43:37.656): `join reveal room=room-1789983698 player=3` — the
  only reveal in the file, yet both checks reported ✅.
- A survivor has **never** logged `join reveal player=1` in any
  recorded run — every historical `player=1` reveal is on
  `instance-1.log` / `instance-1-rejoin.log` (the joiner's own
  process). So `rejoin-1` has always been proxied by `rejoin-3`'s
  line, and the real creator-rejoin failure is masked.

Four independent root chains (below). Each run that fails fails both
reveal checks together (2, 3), or fails earlier on the leave ghost (4).

## Root cause

1. **Creator rejoin never re-identifies — roster ghost blocks `taken`.**
   After the creator is SIGKILLed, its labeled `PlayerNo(1)` slot
   survives because `despawn_on_leave` only removes a slot once the
   peer is **absent from the freenet roster**
   (`src/clicker/bevy_systems/despawn_on_leave.rs:23-44`), and
   `roster::Roster` has no TTL — its only removal is
   `poll_roster.rs:24-33` on a drained `PeerConnected` refcount.
   `resolve_player` then drops the rejoined creator (a *new* peer id)
   because player 1 is already occupied:
   `src/clicker/bevy_systems/resolve_player.rs:44-51` (`taken` guard).
   Proof: `rooms-rejoin-20260921-092745/instance-2.log:3254-3255`
   (09:30:58.169) `join commit received joiner=1` +
   `pending join armed peer=12D3KooWHoeQv9…`, then no `label_slot`
   for that peer, no reveal, and `players=1,2,3` to the end. The ghost
   also carries the old `p1=16`, so `rejoin-restores` looks green.

2. **`JoinCommit` race — `clear_pending` releases the join before
   `decide_join_commit` sends it.** `decide_join_commit` bails when
   `JoinPending` is cleared (`src/clicker/bevy_systems/decide_join_commit.rs:18`),
   but `clear_pending` clears it on quiet (`QUIET_SECS = 1`,
   `src/lobby/constants.rs:3`) while the commit path needs
   `all_synced || 8s cap` (`decide_join_commit.rs:58-71`,
   `JOIN_COMMIT_CAP_SECS = 8`, `src/lobby/constants.rs:6`). Quiet can
   win → the rejoiner never broadcasts a commit → survivors never arm
   pending. Proof: `rooms-rejoin-20260921-092745/instance-3-rejoin.log:257`
   (09:30:04.801) `join ready: quiet roster settled … peers=2` with no
   `join commit sent` anywhere in that file; the survivor's
   `join commit received` count for joiner=3 is 0.

3. **Move gossip labels the rejoiner before the commit arrives.**
   Even when the commit lands, the survivor has usually already
   direct-labeled the rejoiner from position gossip; `spawn_pending_join`
   then skips the numbered slot and no reveal prints:
   `src/clicker/bevy_systems/spawn_pending_join.rs:29-40`
   (`resolved = true`), `reveal_on_join.rs:30-35`
   (`numbered.is_some()` → remove marker, no log). Proof:
   `rooms-rejoin-20260921-093159/instance-2.log:1657` (09:34:02.023)
   `join commit received joiner=3`, and the next
   `pending join armed` is only `:2295` for joiner=1 — nothing armed
   for joiner=3.

4. **SIGKILL survivor ghost (leave gate).** Same removal gap as (1),
   now on the leave path: after app3 is killed the survivors keep
   `players=1,2,3` past the 60s `leave-owners` budget. Proof:
   `rooms-rejoin-20260921-093612/instance-2.log:3627` (09:39:58.630)
   `sync … players=1,2,3 global=46`, and the test panics at
   `tests/e2e_local/rooms_rejoin.rs:167`
   (`leave-owners-2: wrong resolved player count`).

### Staleness stance (decided)

A peer known dead must stop counting as present within **≤30s**. 300s
is far too long; a dead peer must not survive in a resolved roster or
a labeled slot beyond one announce cycle.

- Current: `src/constants.rs:1` `STALE_ENTRY_SECS = 300`; steady
  announce 30s (`src/discovery/run.rs:17,549-553`); the split/bridge
  re-`Put` interval is also 30s (`bridge_tick.rs:7-8`,
  `directory_bridge_tick.rs:6`); `LEAVE_GRACE_SECS = 15`
  (`src/clicker/constants.rs:7`). 300s staleness **and** the 30s bridge
  are both too long for a dead peer to linger.
- `STALE_ENTRY_SECS` only gates dial targets / `expected`
  (`hint_union.rs:46`, `peer_hint_store_prune.rs:5`, `run.rs:1037/1095/1155/1267`,
  `absorb_roster.rs:68`) — it never removes a `roster::Roster`
  presence. Lowering it alone does **not** fix the ghost.
- Decision **1B**: keep dial-hint staleness long, add a short
  **presence TTL ≤30s** plus liveness-driven removal (disconnect /
  last-seen) for `roster::Roster` and labeled slots. Do **not** lower
  `STALE_ENTRY_SECS` to 30 globally (steady announce is also 30s →
  zero margin, live peers would churn).

## Fix

Landed (Phase 1–2, this session):

- **P1 — offset-scoped assertion.** `tests/e2e_local/rooms_rejoin.rs`
  now captures the survivor log offset at the rejoin click and asserts
  `reveal_since(survivor_log, pos)` (`rooms_rejoin.rs:352`), so
  `rejoin-1` is a real signal instead of a stale `rejoin-3` line.
- **P2/P6 (1B) — presence TTL.** New
  `src/clicker/bevy_systems/tick_presence.rs` tracks owners that
  emitted `PeerDisconnected` and removes them from `roster::Roster`
  after `PRESENCE_TTL_SECS = 15` (`src/clicker/constants.rs:8`) unless
  real activity (`Message`/`Gossip`) resumes. Synthetic
  `PeerConnected` (from `poll_expected`) deliberately does **not**
  clear the suspect — that is what let the ghost survive before.
  `despawn_on_leave`'s 15s grace then frees the labeled slot, so a
  dead peer is gone within ~30s and cannot block the rejoiner's
  `resolve_player` `taken` guard. Unit tests:
  `tick_presence::tests::{disconnected_peer_removed_after_ttl,
  liveness_clears_the_suspect}`.
- **P3 — commit-before-clear.** `decide_join_commit`
  (`src/clicker/bevy_systems/decide_join_commit.rs:8`) no longer
  requires `JoinPending`; it keys off `JoinClock.clicked_at` +
  `ActiveLobby`, so `clear_pending` releasing the loading gate can no
  longer starve the `JoinCommit` broadcast. Fast test:
  `decide_join_commit::tests::commits_after_join_pending_cleared`.
- **Decision-log instrumentation.** `resolve_player` (`defer-to-reveal`
  / `direct-label`, deduped), `reveal_on_join` (`revealed` /
  `numbered-skip` / `fail-deadline`), `decide_join_commit`
  (`commit: sent`), `clear_pending` (`join: clear … committed=`), and
  `tick_presence` (`presence: dead peer removed`) write to
  `CLICKER_DECISION_LOG` (`sync-N.log`). A first cut also logged every
  `resolve: taken` hit; because the buffered `Move` is re-processed each
  frame it produced 452,608 lines in `sync-2.log`, so `taken` is now
  silent and `defer` is deduped by the existing `PendingReveal` marker.
  Volume is back to ~1.5k lines per run.
- **`tick_presence` constraints.** The dead-owner set is a
  `Local<HashMap<u64, f64, RandomState>>` (not a `Resource`) to satisfy
  lele `E028`; the file is `tick_presence.rs` (`E017`) and
  `PRESENCE_TTL_SECS` lives in `src/clicker/constants.rs` (`E001`);
  synthetic `PeerConnected` is excluded from liveness so
  `poll_expected`'s re-publish cannot keep a dead peer alive.
- **Log hygiene** — `17-menu-double-despawn.md` (3A): only parentless
  menu roots are despawned, removing the `bevy_ecs` invalid-entity
  spam so the decision trail is readable.

Deferred (P4, handshake-first ordering) — two attempts, both reverted:

1. Arming `JoinGate.pending` on `WantJoin` in `answer_join` defers
   every synthetic peer in the `testing::Mesh` harness (it never sends
   `Move` for them) and broke ~8 fast tests.
2. Deferring in `resolve_player` for any sender in `gate.expected`
   (plus a `claim_slot` ghost-reclaim) fixed `rejoin-3` but
   ping-ponged on the creator: `resolve_player` re-processes the
   **stale `Move` event of the dead peer** every frame (events are
   `take_all`/`extend`), so the dead peer reclaims `PlayerNo(1)` from
   the live rejoiner, whose next `Move` reclaims it back — an endless
   swap that dropped `rejoin-3-owners-2` to 2. (`claim_slot` also had to
   tombstone the ghost's `ClickCounter` before despawn to avoid losing
   its score; it still ping-ponged.)

   Root of both: `p2p::Events` is never drained, so any system that
   keys off a buffered event re-fires it forever. A correct P4 needs
   the move/label path to be event-once (or the mesh harness to model
   it) before the ghost can be reclaimed safely.

Residue from those attempts, cleaned up: `join_gate_arm_pending` had
gained peer-level dedup (`held.peer == entry.peer`) for the `answer_join`
arming; it was inert once `answer_join` reverted, so it was reverted to
the joiner-only rule. A fast test `tests/offline/rejoin/dead_peer_frees_its_slot.rs`
was written and deleted — the `testing::Mesh` harness keys rosters with
`[id; 32]` (see below), so it could not exercise roster-level removal.

Other open sub-slice: `poll_roster` re-adds a removed peer from
buffered `PeerConnected` events (re-added by `send_want_join.rs:33` /
`send_sync_req.rs:35`), so `tick_presence`'s roster removal is undone
unless it runs after `poll_roster`. Fast-suite unit tests cover
`tick_presence`; a mesh-level test is blocked because
`testing::mesh_of::link_rosters` uses `[id; 32]` roster keys instead of
`blake3(peer_id)`.

## Verification

- Fast suite green: `lele:clippy` / `fmt` / `nextest` (386/386) /
  `lint` / `bevy-lint` / `taxonomy_check`.
- Slow gate after the landed slices (`.local-run/rooms-rejoin-20260921-13*`,
  `telegram_bot/.local-run/rooms_rejoin-*`):
  - `phase5-run1`: 43/44 — only `rejoin-1` reveal RED; `rejoin-3`
    reveal ✅ (real, offset-scoped).
  - `phase5-run2`: 42/44 — `rejoin-3` + `rejoin-1` reveal RED.
  - All functional checks pass in both: owners 3/3, `rejoin-restores`,
    `creator-rejoin-agreement`, `leave-owners`, 500ms budget (58–73ms).
- Compared with the pre-slice baseline (3/5 RED, both checks, false
  passes), the gate is now **honest**: `rejoin-3` is a real signal and
  `rejoin-1` deterministically REDs on the still-open ghost.

## Open / next slice

- **Correct P4.** Needs an event-once move/label path (or mesh-harness
  support) before the ghost can be reclaimed; see the two reverted
  attempts above.
- **`rejoin-3` residual flake.** Still ~1/2; the `JoinCommit`-vs-Move
  ordering remains the race (P3 removed the *never-committed* path but
  not the *commit-after-Move* path).
- **`poll_roster` re-add.** Removing a peer from the roster is undone
  by buffered `PeerConnected`; either drain events or order
  `tick_presence` after `poll_roster`.
- **Mesh-harness keys.** `testing::mesh_of::link_rosters` should use
  `blake3(peer_id)` so roster-level behavior is fast-testable.
