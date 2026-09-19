# MAINNET PROBLEMS 9 — rooms-rejoin observability: dead cursors, leave that does not stick, slow score convergence

Follow-up to `MAINNET_PROBLEMS_8.md`. This documents artifacts observed in
the **rooms-rejoin slow gate** after the synchronized-join-handshake work
(the run that PASSED on CI-ish `master@0e15be1`). The e2e passing is
misleading: its assertions are snapshot-based and do not catch a leaver
that never disappears, nor a client that lags ~41s behind on score.

**No product `src/` change in this session** — this file is the record.
Run binary: `master@0e15be1` (`feat(clicker): log resolved players and assert
them in the rejoin e2e`).

## 0. Where the logs are

The slow gate writes one dir per run under the (gitignored) run root:

- Run dir for this evidence:
  `clicker/.local-run/rooms-rejoin-20260919-113027/`
  - `instance-1.log` — creator, before it is killed for the creator-rejoin phase
  - `instance-2.log` — survivor, spans the whole run (best single log)
  - `instance-3.log` — before the app3 leave/rejoin phase
  - `instance-1-rejoin.log`, `instance-3-rejoin.log` — rejoined processes
  - `instance-4.log` — join-then-leave-during-loading probe
  - `raw.mp4`, `clip.mp4` — screen capture (do not commit)
- Task stdout captured on this machine: `/tmp/opencode/e2e5.log`
  (and `/tmp/opencode/e2e3.log`, `/tmp/opencode/e2e4.log` for the two
  earlier failing runs). `/tmp` may be wiped; the instance logs are the
  primary source.

Regenerate + find the newest run:

```bash
cd clicker
devenv tasks run freenet:run-rooms-rejoin 2>&1        # ~4-5 min, 3 xterms + mainnet
ls -dt .local-run/rooms-rejoin-* | head -1
```

Useful greps (strip ANSI first):

```bash
R=.local-run/rooms-rejoin-20260919-113027
sed 's/\x1b\[[0-9;]*m//g' $R/instance-2.log | grep 'sync lobby='
grep -an 'players=1,2,3,4' $R/instance-2.log
```

## 1. Leave does not stick — instance 4 auto-rejoins (dead cursor)

`phase_leave_load` joins instance 4 then clicks leave. The log shows it
reaching the menu, then **returning to the room on its own**, which leaves
`player=4` resolved on every survivor for the rest of the run.

Evidence (`instance-4.log`, ANSI-stripped):

```
instance-4.log:97   11:34:18.016  sync lobby= p1=0 p2=0 p3=0 players=4 global=0   (menu, lobby cleared)
                    11:34:23.298  directory listed rooms
instance-4.log:600  11:34:31.111  sync lobby=room-1789817443 p1=15 p2=15 p3=15 players=1,2,3,4 global=45
```

It was in the menu ~11:34:18–11:34:27, then in-room again at 11:34:28
without any UI action. `player=4` then persists to the end on all peers:

```
instance-1-rejoin.log:739  11:34:28.325  sync lobby=room-... players=1,2,3,4 global=45
instance-2.log:3329        11:34:28.544  sync lobby=room-... players=1,2,3,4 global=45
instance-3-rejoin.log      11:34:47.749  sync lobby=room-... players=1,2,3,4 global=45
instance-2.log (tail)      11:34:49.865  sync lobby=room-... players=1,2,3,4 global=45
```

**Hypothesis.** `leave_room` clears only local state (`ActiveLobby`,
`GlobalCounter`, `PendingClicks`, tombstones) and sets `State::Menu`
(`src/lobby/bevy_systems/leave_button.rs:27-29`, `src/lobby/leave_room.rs`).
It does not clear the `RoomRx` watch nor `SelectedRoom`.
`apply_room` runs every `Update`, re-reads the stale watch value, sees
`**rooms.active != room`, and calls `lobby::join_room` again
(`src/lobby/bevy_systems/apply_room.rs:13-27`). So a deliberate leave is
undone on the next frame.

Note the e2e `leave-during-loading-4` soft check passed because the menu
*did* appear briefly; it never asserted the leave stayed. Also the dead
cursor survives because a peer that stays connected is never
`PeerDisconnected`, so `despawn_on_leave` (roster-based) keeps it.

## 2. Slow score convergence — one client at 45, another stuck at 30

After the 15-clicks-each drive, the three clients did not agree for a long
time. First `global=45` per log:

```
instance-3.log:971   11:32:14.778  first global=45
instance-1.log:2150  11:32:25.219  first global=45   (+10.4s)
instance-2.log:2150  11:32:55.754  first global=45   (+41.0s vs instance-3)
```

While instance-3 already showed 45, instance-2 still showed 30 with
`p3=0`:

```
instance-2.log:2113  11:32:54.740  sync ... p1=15 p2=15 p3=0  players=1,2 global=30
```

So instance-2 was missing **all 15 of player 3's clicks for ~41s** while
identity was already resolved. This is score propagation (snapshot/delta
merge path), not identity or roster: `players=1,2,3` was already present.

The e2e only passed because `agree_within` polls up to
`LAG_AGREE_TIMEOUT_SECS = 60` and this run converged at ~41s (≈19s margin).
`owners`/`resolved` assertions were satisfied, so nothing surfaced.

## 3. Roster churn windows

`players=<self only>` with the room active shows periods where a survivor
saw nobody else:

```
instance-2.log:553   11:31:08.842  sync lobby=room-... players=2 global=0
                     11:31:22.975  sync lobby=room-... players=2 global=0   (~14s alone)
instance-3.log       11:31:36.468  sync lobby=room-... players=3 global=0
                     11:31:37.483  sync lobby=room-... players=3 global=0
```

This is expected around the app3 stop / creator stop phases, but the
duration and the fact instance-2 was alone with the room active is worth
watching as a symptom of the same leave/rejoin churn.

## 4. Own cursor persists in the Menu

`sync lobby=` (empty lobby) still reports the local `PlayerNo`:

```
instance-4.log:97   11:34:18.016  sync lobby= p1=0 p2=0 p3=0 players=4 global=0
instance-2.log:116  11:30:58.812  sync lobby= p1=0 p2=0 p3=0 players=2 global=0
```

`spawn_cursor` runs at `Startup` (before any room) and assigns
`PlayerNo(own)`; `despawn_leave` is the only remover on `OnEnter(Menu)`.
Whether this is by design or a leak matters for the visual "dead cursor"
report; confirm `despawn_leave` actually runs on the leave path
(`src/lobby/plugin_build.rs`, `OnEnter(AppState::Menu)`).

## 5. Why the e2e did not catch any of this

- `assert_owners` counts **resolved** `PlayerNo` at phase boundaries and
  waits for the expected count; it never asserts that a leaver's cursor
  **disappears**, so a re-joined/never-left player 4 looks like a valid
  member.
- `agree_within` has a 60s budget; 41s of divergence passes.
- There is no reveal-skew measurement (log lines are not timestamped in a
  machine-friendly way for the game; the ANSI lines above do carry
  timestamps from `tracing`).

## 6. Next work (suggested order)

1. **Make leave stick.** Clear the room watch / `SelectedRoom` on leave, or
   make `apply_room` ignore a room equal to the last deliberately-left
   room. Red fast test: leave, run several `Update`s, assert `State::Menu`
   stays and own `PlayerNo` entity is gone.
2. **Assert leaver removal in the e2e.** After `ui_leave_room`, wait for
   survivors' resolved-player set to drop the leaver; fail hard if it does
   not. This requires a way to know the leaver's logical id
   (`instance-4` uses `--own-id 4`).
3. **Bound convergence time.** After the drive, assert all three `sync`
   lines agree within a tight budget (e.g. 10-15s), so a 41s lag fails.
4. **Investigate score propagation lag** on the merge path
   (`send_sync_*`, `sync_global`, `publish_snapshot`, `absorb_snapshot`,
   `apply_delta`): why did instance-2 miss player 3's clicks for ~41s after
   `players=1,2,3` was already satisfied?
5. **Own-cursor lifecycle in Menu.** Confirm `despawn_leave` removes the
   own cursor; if not, decide whether it should.
6. Optional: structured/timestamped reveal logging to assert reveal skew
   across clients.

## 7. Scoreboard for this run

- Slow gate: **PASS** (`1 passed`, 267.8s), `master@0e15be1`.
- But visible artifacts: dead `player=4` cursor on all survivors to the
  end; ~41s score-convergence lag (45 vs 30) survived only because the
  budget is 60s.
- Earlier runs on the same feature: `e2e3` failed at
  `rejoin-1-owners` (5 raw owners — stale roster peer ids), `e2e4` failed
  at `phase converge: no baseline agreement` (flaky convergence).
