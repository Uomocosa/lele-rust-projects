# 18. `ui_leave_room` false negative (leave succeeds, menu marker never logs) — OPEN

Related: `mainnet_problems/12-leave-sticks-dead-cursor.md` (leave path), `mainnet_problems/16-rejoin-reveal-log-flake.md` (same e2e suite).

## Symptom

`phase_leave_load` fails with `phase leave during loading: "instance 4
never left via the leave button"` even though the instance did leave:
the app returns to the menu and its `lobby=` goes empty.

## Evidence (logs)

- Run `clicker/.local-run/rooms-rejoin-20260921-131011/instance-4.log`
  (the failed phase-2 run): from `13:14:33.496` the survivor's own log
  shows the menu state —
  `sync lobby= p1=0 p2=0 p3=0 players= global=0` — repeating to the end
  of the file, i.e. the leave **succeeded**.
- Task stdout `/tmp/opencode/rejoin_runs/phase2-run1.log`:
  `phase leave during loading: "instance 4 never left via the leave button"`.

## Root cause

`ui_leave_room` (`tests/e2e_local/rooms_rejoin.rs:465`) clicks the
leave spots and waits for `menu_since(log, pos)` (`:449`), which looks
for the line `directory listed rooms`. But `poll_directory` only emits
that line when the room list **changes**
(`src/lobby/bevy_systems/poll_directory.rs:47-54`: `if same { return; }`).
After the initial listing the directory is usually unchanged, so the
marker is never printed inside the 15s window and the helper returns a
false negative — a test-assertion bug, not a product leave failure.

## Fix

None yet. Options: assert on app state (`lobby=` empty / `LeftRoom` set
for the room, mirroring `leave_sticks_across_apply_room`), or add a
dedicated `leave: menu shown` log on the `OnEnter(Menu)` transition and
key `menu_since` off that.

## Verification

- Observed once (`phase2-run1`, 2026-09-21). Other runs in the same
  session passed `leave-during-loading-4` (e.g. `phase5-run1`/`run2`),
  so it is intermittent.

## Open / next slice

- Replace the `directory listed rooms` marker with a leave-specific
  signal; add a fast test for the helper's offset scan.
