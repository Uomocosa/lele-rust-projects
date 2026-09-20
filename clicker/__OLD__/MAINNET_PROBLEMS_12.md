# MAINNET PROBLEMS 12 — slow RED with decision logs: fail deadline despawns a healthy labeled cursor

Follow-up to `MAINNET_PROBLEMS_11.md`. Slow gate
`freenet:run-rooms-rejoin` on the frozen-gate-fix tree: **RED** —
`phase converge: no baseline agreement`
(run `.local-run/rooms-rejoin-20260919-182713/`, 158s).
The frozen-slot fix itself worked (creator phase passed, all three
drives completed). This run also produced the first usable per-instance
decision logs (`sync-{1,2,3}.log`, ~41–44KB each).

## Evidence (same defect as `_11`, rotated victim, now with logs)

```
instance-1.log tail  p1=16 p2=16 p3=15 players=1,2,3 global=47
instance-3.log tail  p1=16 p2=16 p3=15 players=1,2,3 global=47
instance-2.log tail  p1=16 p2=16 p3=0  players=1,2   global=32
```

Instance-2 timeline from `instance-2.log` + `sync-2.log` (403 lines):

- `players=1,2,3` for 31s; `label_slot` resolved `player=3` at 18:28:09.
- At 18:28:40 `players` drops to `1,2` permanently. Same second:
  `WARN clicker: join failed: no identity before the fail deadline
  peer=12D3KooWCD4npu7NTjY9PpSS1ahVfpgiduQazzqm6Bs3LHNvKx2K`
  (player 3's transport).
- After that, every received `(NetworkId(3), 15)` parks:
  `sync: entry parked pending id=NetworkId(3) count=15` (31×) —
  transport healthy, merge has nowhere to put it.

## Root chain (all current source)

1. `gate.pending` entry for the peer arms early (join handshake).
   `resolve_player.rs` labels the skeleton directly on gossip `Move`
   but never disarms the pending entry — stale entry outlives the
   resolution it waited for.
2. `spawn_pending_join.rs:28-40` matches the existing slot by `Owner`
   and inserts `PendingReveal { player: None }` (remote joiner) **onto
   the already-labeled slot** — no `PlayerNo` check.
3. `reveal_on_join.rs:43-49`: at `fail_at` with `player: None`, it
   despawns the healthy numbered cursor, pushes the peer to
   `gate.absent`, and drops the pending entry.
4. Recovery is structurally impossible afterwards: `spawn_on_join`
   refuses absent peers (`held`), `drain_pending`/`find_slot` need an
   existing slot, so `SyncAck` entries park forever (`_11`'s
   never-merges signature).

Note the fail-deadline log line names the origin precisely — the
`definition-tdd` LOG FIRST requirement is already satisfied by the
existing `warn!` + `DecisionLog` lines; no new logging needed.

## Next slices (TDD, in order)

- **R1 — resolve disarms.** When `resolve_player` labels `Owner` X,
  drop `gate.pending` entries mapping to X. RED: armed pending + gossip
  `Move` labels the slot → assert pending no longer holds the peer.
- **R2 — never re-arm a labeled slot.** `spawn_pending_join` skips
  entities already carrying `PlayerNo`. RED: labeled slot + stale
  pending entry → run `spawn_pending_join` → assert no `PendingReveal`
  on it; run `reveal_on_join` past `fail_at` → assert entity alive and
  peer not absent.
- Then re-run the slow gate as the only oracle.

## Collateral (decided, not started)

- `assert_slot_frozen` (harness, test-only, uncommitted): was hard
  `first == 0 && all-equal`; the create click legitimately counts
  (`p1=1` on the first post-click sample, `_5 §11` allows ≤1), so the
  18:19 run died flakily at `create-frozen-1`. Relaxed to
  `first <= 1 && all-equal` (constancy + join-click allowance).
- Decision-log pipeline verified working end to end
  (`XtermSpec.decision_log` → `CLICKER_DECISION_LOG` → `drain_decisions`
  → `sync-N.log`). Earlier empty files = no sync events pre-connection
  or early kills, not a broken pipe.
- User's 16:xx runs (pre-LeftRoom): baseline converged at 45 but show
  `players=1,2,3,4` — the instance-4 never-leaves ghost. Covered by the
  pushed `LeftRoom` guard; watch for its absence in the next gate.
