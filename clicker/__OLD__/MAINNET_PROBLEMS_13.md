# MAINNET PROBLEMS 13 — five-gate campaign: every problem, with log pointers

Method: run `freenet:run-rooms-rejoin` 5× on the R1/R2 tree
(`spawn_pending_join` skips labeled remotes, `reveal_on_join` never kills
labeled slots, frozen-gate `≤1`, 500ms measurement). After each run, read
the instance logs + `sync-N.log` decision files and register EVERY
deviation here with exact log pointers before starting the next run.
Per TDD rule 5 (stop on first slow red → fast test), each finding names
the fast slice it demands; no product fixes inside this file.

Tree: uncommitted R1/R2 + frozen-gate fix on top of `c93b142`.
Gate binary rebuilt per run via `build_game()`.

## Run 1 — PASS (run `rooms-rejoin-20260920-154053`, gate `/tmp/opencode/gate1.log`)

Fully green checklist, zero ❌. Baseline `p1=16 p2=15 p3=15 global=46`
on all three; `baseline-converge-ms: 434ms ≤ 500ms` — first budget
measurement holds. Both rejoins restore exact state; creator-survives,
leave-during-loading-4 all ✅.

- **F1 (residual, known): zero-count player-4 ghost on survivors.**
  Tails: `instance-2.log`, `instance-1-rejoin.log`,
  `instance-3-rejoin.log` end `players=1,2,3,4 global=46`
  (16+15+15=46, slot 4 contributes 0). Leaver itself is clean:
  `instance-4.log` tail `sync lobby= (empty) players= (empty)` —
  LeftRoom holds, no rejoin. Remaining half needs a leave broadcast
  (no `PeerDisconnected` ever fires on deliberate leave).
  Demands fast slice: leaver-removal e2e assert (`_9 §6.2`).
- **F2 (watch, not a defect): late `parked pending` with correct finals.**
  `sync-{1,2,3}.log` show 2–12 parked entries in each file's last
  recorded minute — e.g. `sync-1.log`: 6× `(3,15)` — yet every final
  is exact. Parking without later `merge_entries` is absorbed safely
  (click path / max-merge cover it). Becomes a defect only if a final
  ever diverges; no fast test until then.
- **F3 (pre-existing cosmetic): `Entity despawned` double-despawn
  warnings**, 13–30 per instance log (menu-teardown vs queued commands,
  `_5 §11` family). Unchanged, no gameplay effect observed.
- **Absent killers:** `join failed` 0× everywhere, absent-marks 0×,
  `p3=0`-forever 0×. The `_12` fail-deadline defect did not recur.

## Run 2 — RED at `create-ready-1` (run `rooms-rejoin-20260920-154557`, gate `/tmp/opencode/gate2.log`)

`join create-ready-1 not ready within 60s of click`, 85s in. Only
`instance-1.log` exists (creator never became ready, no joiners spawned).

- **F4 (known tail, Freenet layer): roster `Get` stalled on a fresh key.**
  `instance-1.log`: `discovery: room resolved room-1789919174`
  (15:46:14) then `discovery: roster connect failed, retrying
  error=response timed out` at 15:46:24, 15:46:39, 15:46:54, 15:47:09 —
  not one roster fetch succeeded in 60s+. Consequence logged at
  15:46:44: `join abandoned: roster never resolved before the
  alone-cap`, so `clear_pending` never fires and no `join ready` line
  ever prints (only `sync lobby=room-1789919174 p1=1 … players=1` ticks).
  Same `_5 §6/P7` + `_6 §4` Get-stall family on a churned ring; demands
  the still-open structural slice (roster `Get` off the click path,
  `_5 §9`). Not reproducible fast by construction — stays a
  slow-gate-only row.
- Side confirmation: `sync-1.log` is 0 bytes — empty decision log means
  "joined nothing, no sync events", settling the empty-file question
  from the campaign prep. Pipeline itself verified working in run 1.

## Run 3 — PASS (run `rooms-rejoin-20260920-155036`, gate `/tmp/opencode/gate3.log`)

Zero ❌. Baseline exact; `baseline-converge-ms: 382ms ≤ 500ms`.
`join failed` 0×, absent-marks 0× on all five logs; despawn-warning
counts identical shape to run 1 (30/30/13/30/13).

- **F1 variant: ghost carries the join click.** Tails on
  `instance-2.log`, `instance-1-rejoin.log`, `instance-3-rejoin.log`:
  `players=1,2,3,4 global=47` while 16+15+15=46 — slot 4 kept
  instance-4's join click (1pt). Same leave-broadcast residual as run 1,
  one point heavier. `instance-4.log` tail empty lobby again (leave
  sticks ✅).

## Run 4 — RED on the budget only (run `rooms-rejoin-20260920-155748`, gate `/tmp/opencode/gate4.log`)

476s. Every functional check ✅ (both rejoins exact, leave phases,
owners) — the single ❌ is `baseline-converge-ms: 731ms > 500ms`, and
`phase_finish` (rooms_rejoin.rs:1360) promotes any soft ❌ to a fatal
`phase finish` panic, so the run is RED on the budget alone.

- **F5 (new, app sync path): one round-trip too many for the late
  labeler.** First `players=1,2,3 global=46` ticks: instance-2 at
  16:03:30.323, instance-3 at 16:03:30.533, instance-1 at
  16:03:30.565. Instance-1's own last-8 sync lines show the transition
  (`p1=16 p2=15 p3=0 players=1,2 global=46` ×7 → `p3=15 players=1,2,3`
  on the 8th; note global=46 throughout — tombstone retention already
  knew the total, only the live slot lagged). So post-drive agreement
  waited ~0.7s on instance-1 labeling/merging p3's entries: one
  SyncReq/heartbeat round-trip. Candidate fast slices (not started):
  label-from-`SyncAck` directly, or tighten the heartbeat trigger on
  post-drive growth. Alternative: relax budget to 1s — user's call.
- Frozen-fix validation: `create-frozen-1 p1 samples: [1,1,1,1,1,1,1]`
  ✅ (would have REDDed under the old `==0` rule).

## Run 5 — RED, same F4 signature (run `rooms-rejoin-20260920-160633`, gate `/tmp/opencode/gate5.log`)

`join create-ready-1 not ready within 60s`, 85s in. `instance-1.log`:
room resolved 16:06:50, then roster `response timed out` at 16:07:00,
16:07:15, 16:07:30, 16:07:45; `join abandoned: roster never resolved
before the alone-cap` at 16:07:20. Byte-identical shape to run 2. No
new information; F4 confirmed recurrent (2/5 gates).

## Campaign verdict — 2/5 PASS, three distinct problems

| Run | Result | Deciding problem |
|---|---|---|
| 1 (`…154053`) | PASS, converge 434ms | — (F1 ghost + F2/F3 watches only) |
| 2 (`…154557`) | RED @ creator-ready | **F4** roster-Get stall |
| 3 (`…155036`) | PASS, converge 382ms | — (F1 ghost w/ join click) |
| 4 (`…155748`) | RED @ phase-finish | **F5** converge 731ms > 500ms |
| 5 (`…160633`) | RED @ creator-ready | **F4** roster-Get stall, same shape |

Ranked by gate cost:

1. **F4 — fresh-key roster `Get` stall (killed 2/5).** Freenet-layer
   tail, known since `_5 §6/P7`. Biggest available win is the
   still-open structural slice: roster `Get` off the click path
   (`_5 §9`: hints → dial → PEX → targeted syncs, contract union in
   background). No fast test possible by construction.
2. **F5 — post-drive converge 382–731ms across runs (killed 1/5).**
   App-layer, one sync round-trip for the late labeler. Slices:
   label-from-`SyncAck`/tighter heartbeat trigger, or accept a 1s
   budget. Note the budget mechanics: `phase_finish`
   (rooms_rejoin.rs:1360) promotes any soft ❌ to fatal, so the
   "soft" 500ms check is effectively hard — keep that in mind when
   setting the number.
3. **F1 — player-4 zero-count ghost on survivors (present 3/3 full
   runs, never fatal).** Leaver side fixed (empty-lobby tails);
   survivor side needs a leave broadcast. Demands the leaver-removal
   e2e assert first (`_9 §6.2`).

Not recurring in 5 runs: the `_12` fail-deadline kill (0× `join
failed`), `p3=0`-forever (0×), rejoin state loss (exact restores
everywhere they ran). R1/R2 hold.
