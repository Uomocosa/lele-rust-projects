# Mainnet problems — index

> All observed clicker mainnet failures, their fixes, and open slices
> live in `mainnet_problems/` (one file per problem). Start here.

## How to contribute (agents: read before adding)

1. **One file per problem** in `clicker/mainnet_problems/`, named
   `NN-short-slug.md` (next free number, lowercase, hyphens). One
   problem = one symptom + one root chain. If a run surfaces two
   unrelated defects, write two files, not one.
2. **Copy the template** (§Template below). Every file carries:
   status line, provenance (`Follow-up to:` old sources or prior
   files), symptom, evidence with log pointers, root cause, fix with
   `src/path:line`, verification (fast tests + slow-gate run id +
   result), open/next slice.
3. **Status words** (in the `# N. Title — STATUS` header, exactly one):
   `SOLVED` (fix landed + slow gate green), `MITIGATED` (workarounds
   hold, root cause open), `OPEN` (no fix), `BY DESIGN` (won't fix,
   gate constrains around it), `LIVING DOC` (rig/process notes, no
   defect). Flip the word when the state changes — never leave a
   SOLVED header on an OPEN problem.
4. **Log pointers** are mandatory for slow-gate evidence. Format:
   `.local-run/<run-dir>/instance-N.log:<line>` plus the `tracing`
   timestamp where the old file quotes one, e.g.
   `.local-run/rooms-rejoin-20260919-113027/instance-4.log:600
   (11:34:31.111 ...)`. Run dirs are gitignored and `/tmp/opencode/`
   logs may be wiped — if the run is gone, write `(run dir deleted —
   shape only)` instead of inventing lines.
5. **Link the fix**, not just prose: `src/lobby/left_room.rs`,
   `tests/turmoil/connect/*.rs`, fast test name (was RED → GREEN),
   slow gate (`freenet:run-rooms-rejoin`, run dir + PASS/RED + seconds).
6. **Update this index table** in the same commit as the new file.
   Never edit a `mainnet_problems/` file and skip the table.
7. **Layer rule** (`MAINNET_PROBLEMS_10.md`, kept in
   `10-join-handshake-welcome.md`'s neighbors): Freenet = rooms +
   occupants seed only; all dials/connections/updates = libp2p.
   Never attribute a gameplay delay to the ring. Room converge bar:
   **≤500ms same-host** (`13-converge-budget-500ms.md`).

## Problems

| # | File | Status | One-line summary |
|---|---|---|---|
| 1 | `mainnet_problems/01-cold-start-partition.md` | MITIGATED | Fresh-key concurrent `Put`s split replicas; stagger + late-Get pull work around it |
| 2 | `mainnet_problems/02-stale-roster-dead-ports.md` | MITIGATED | Stable ids, ephemeral ports → ghosts; burst announce + 300s staleness + unique test keys |
| 3 | `mainnet_problems/03-bridge-notification-loss.md` | SOLVED | Bridge wait dropped `UpdateNotification`s; single `poll()` drain |
| 4 | `mainnet_problems/04-position-heartbeat.md` | SOLVED | Idle mice = no resolution; 5s heartbeat, 15 Hz fast path |
| 5 | `mainnet_problems/05-full-mesh-data-plane.md` | BY DESIGN | Unicast game traffic needs full mesh; transitive sync closes stars for scores only |
| 6 | `mainnet_problems/06-resolution-gate.md` | SOLVED | Gate asserts `resolved>=2`, not just connection |
| 7 | `mainnet_problems/07-dial-storm-tiebreak.md` | MITIGATED | Simultaneous multi-addr dials flap; lower-id-dials tie-break + dial-once per gossip id |
| 8 | `mainnet_problems/08-roster-get-stall.md` | MITIGATED | Q1 hint-union dial-before-Get landed (`15`): stalls survive the gate (0/5 kills); budget/reveal tails open |
| 9 | `mainnet_problems/09-instant-join-path.md` | MITIGATED | P0–P5: loading gate, DirectoryLive, resolve quantum, pre-switch links, event starvation |
| 10 | `mainnet_problems/10-join-handshake-welcome.md` | SOLVED | `WantJoin`/`Welcome` + spectate + 1s-quiet/30s-cap readiness (commit `733fd77`) |
| 11 | `mainnet_problems/11-score-retention-tombstones.md` | SOLVED | Retention rule + tombstones on `SyncAck` + dup-identity + messenger-mismatch fixes |
| 12 | `mainnet_problems/12-leave-sticks-dead-cursor.md` | MITIGATED | `LeftRoom` + R1/R2 disarm; survivor-side leave broadcast still missing (F1 ghost) |
| 13 | `mainnet_problems/13-converge-budget-500ms.md` | MITIGATED | 1 Hz `sync` probe fixed → event-driven; 5/5 runs 50–107ms; gate still red on `16` |
| 14 | `mainnet_problems/14-turmoil-rig-fidelity.md` | LIVING DOC | Turmoil coverage matrix #1–17, lessons, remaining rig work |
| 15 | `mainnet_problems/15-hint-union-dial-before-get.md` | MITIGATED | Q1 dial-before-Get + synthetic-connect dedup; stall kills 0/5, reveal race + budget open |
| 16 | `mainnet_problems/16-rejoin-reveal-log-flake.md` | OPEN | Survivor `join reveal` line missing after rejoin (9/20 baseline); gate REDs on it |

## Retired sources

Old per-session files (moved to `clicker/__OLD__/`) and where each
section now lives:

| Old file | New home |
|---|---|
| `MAINNET_PROBLEMS.md` P1 | `01-cold-start-partition.md` |
| `MAINNET_PROBLEMS.md` P2 | `02-stale-roster-dead-ports.md` |
| `MAINNET_PROBLEMS.md` P3 | `03-bridge-notification-loss.md` |
| `MAINNET_PROBLEMS.md` P4 | `04-position-heartbeat.md` |
| `MAINNET_PROBLEMS.md` P5 | `05-full-mesh-data-plane.md` |
| `MAINNET_PROBLEMS.md` P6 | `06-resolution-gate.md` |
| `MAINNET_PROBLEMS_2.md` F1–F5 | `01`, `07`, `05`, `11`, `05` |
| `MAINNET_PROBLEMS_3.md` | `01`, `07`, `09` (PEX/mDNS/UI/matrix prologue) |
| `MAINNET_PROBLEMS_4.md` iters 0–5 | `09` (iters 0–4 prologue) + `11` (iter-5 tombstone-wins) |
| `MAINNET_PROBLEMS_5.md` §§1–14 | `09`, `10`, `08` (§§6–9), `02` (§7) |
| `MAINNET_PROBLEMS_6.md` | `11`, `14` (rig part) |
| `MAINNET_PROBLEMS_7.md` | `14`, `11` (§3: 51-inflation) |
| `MAINNET_PROBLEMS_8.md` | `14`, `07` (tie-break proof) |
| `MAINNET_PROBLEMS_9.md` | `12`, `13` (§§2–3) |
| `MAINNET_PROBLEMS_10.md` | Layer rule + bar (copied into §How to contribute.7); slices 2–4 → `12`, slice 5 → `13` |
| `MAINNET_PROBLEMS_11.md` | `13` + `08` (whole-run variant) |
| `MAINNET_PROBLEMS_12.md` | `12` (R1/R2) |
| `MAINNET_PROBLEMS_13.md` F1–F5 | `12` (F1), `08` (F4), `13` (F5) |
| `MAINNET_PROBLEMS_14.md` | `08` (dev-profile proof) + `13` (frozen-gate streak → open slice) |
| `TURMOIL_TO_EXPLAIN.md` | `14-turmoil-rig-fidelity.md` (intro + plan a) |

## Template (copy for a new problem file)

```markdown
# NN. <Title> — <SOLVED | MITIGATED | OPEN | BY DESIGN>

Follow-up to: <old file § / prior problem file>.
Related: `mainnet_problems/<nn>-*.md`.

## Symptom

## Evidence (logs)
- `.local-run/<run-dir>/instance-N.log:<line>` (timestamp …):
  quoted lines …

## Root cause

## Fix (with src:line)

## Verification
- Fast: <test names, RED → GREEN>
- Slow: `freenet:run-rooms-rejoin` <run-dir> PASS/RED (<seconds>s)

## Open / next slice
```
