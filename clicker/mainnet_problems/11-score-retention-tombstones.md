# 11. Score retention, tombstones on the wire, identity bugs — SOLVED

Follow-up to: `MAINNET_PROBLEMS_2.md` Finding 5, `MAINNET_PROBLEMS_6.md` (RED + GREEN), `MAINNET_PROBLEMS_7.md` §3 (51-inflation), `TURMOIL_TO_EXPLAIN.md` plan (a).
Related: `mainnet_problems/05-full-mesh-data-plane.md`.

## Symptoms

1. **Total drops when a player leaves.** Converge to (1,5,17), peer-3
   leaves → survivors show 6, not 23.
2. **Diverged survivor stays low.** B partitioned from C pre-leave
   learns `{1:1,2:5}` (6) while A learns `{1:1,2:5,3:17}` (23); naive
   retention keeps them disagreeing.
3. **Duplicate player identity.** Creator rejoin → `global=66`
   (16+16+17+17: stale roster id + new peer id) and `p1=0` elsewhere.
4. **51-inflation (messenger mismatch).** `peer-1: 51 = 1+28+22`,
   `stones={2:22, 3:0}`; two slots labeled 3.

## Evidence (logs)

- Turmoil RED (deterministic, no ring needed):
  `tests/mesh/total_retained.rs` — `assertion left == right failed:
  survivor peer-1 lost the leaver's total, left: 6, right: 23`.
- `_7` §3 slice-1 repro (`tests/mesh/attribution.rs`): pre-fix dump
  shows `slots=[(Owner(1), Some(1), 0), (Owner(hash-peer-2), Some(3),
  17), (Owner(hash-peer-3), Some(3), 17)]` — TWO slots labeled 3.
- Slow gate 1 RED → duplicate identity: `agree_within` failed after
  creator rejoin with `global=66` on one instance
  (`.local-run/rooms-rejoin-*`, dirs deleted).

## Root causes

1. `despawn_on_leave.rs:30-36` parks the score in tombstones but
   `sync_global.rs:9-15` summed **live counters only**.
2. Payload asymmetry: `publish_snapshot.rs:31-61` unions live +
   tombstones, but `absorb_sync.rs:68-82` `snapshot_entries` sent live
   labeled slots only — B can never heal without A's tombstone.
3. `resolve_player.rs` labeled an already-taken `PlayerNo` twice.
4. `find_slot` (`drain_pending.rs`) fell back to the *transport
   sender's* slot for **absolute** `SyncAck` entries — but the sender
   is just the messenger relaying third-party scores. The absolute
   `(3,17)` labeled the messenger's own slot 3 while the true slot
   also reached 17: counted twice.

## Fixes (with src:line)

- `sync_global.rs`: total = Σ labeled slots (max with tombstone on
  overlap) + tombstones with local provenance (`Local<HashSet>`,
  cleared when the map empties; `leave_room.rs:12` keeps
  room-scoping). `unlabeled_snapshot_drops` stays green; `_4` iter-2
  ghost family stays dead.
- `SyncAck` replies union retained tombstones, max per id
  (`SyncCtx` SystemParam bundle + private
  `sync_ctx_snapshot_entries.rs`; `absorb_sync` 8→6 params).
  `merge_entries` unchanged (unknown ids park as absolute
  `PendingClick`s); `keep()` max-merge makes replays no-ops.
- `resolve_player.rs` never labels an already-taken `PlayerNo`
  (`duplicate_player_no_never_labels_twice`); global counts labeled
  slots only (`unlabeled_count_never_counts`).
- Drain fallback applies to non-absolute (delta) pendings only;
  absolute entries with no logical match stay pending (retried every
  update). Regression: `attribution::unlabeled_clicks_park_exact`,
  `relay_labels::prelabel_partition_no_inflation`.
- Collateral: `emit_flash` ∥ `despawn_on_leave` race fixed by
  chaining `(despawn_on_leave, emit_flash)` in `plugin_build.rs`
  (+ removed duplicate `emit_flash` registration).

## Verification

- Fast: 289 nextest green; `total_retained` 2/2 (incl. seed matrix
  `[7,8,9]`), `total_retained_diverged`, attribution + relay cases.
- Slow: `rooms_rejoin` PASS (630s).

## Open / next slice

- Rejoined peer ids still learn slowly on the survivor side (roster
  freshness / dial-failed pruning) — masked, expect back under churn.
- `peer-2 == 23` in `prelabel_partition_no_inflation` documents a
  design limit: never labeled the leaver, no `Move`s arrive post-leave,
  absolutes stay parked per the provenance rule.
