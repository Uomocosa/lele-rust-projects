# POLISH

> **SUPERSEDED by [`POLISH_2.md`](./POLISH_2.md).** This file predates the convergence fix
> (`docs/CONVERGENCE_INVESTIGATION.md`): §1 references files from example_2 that no longer exist
> here and `spawn_box.rs` is in fact used; §2 frames the rollback crate as future work when it is
> already built and integrated. The authoritative path-to-CONCLUDED plan is `POLISH_2.md`. Keep
> this file for history only.

Pre-`FINISHED` cleanup and remaining work for `example_4`. Everything here is either code
cleanup (dead code), the prediction/rollback build, or the final cross-OS/cross-network gate.

[[README]] | [[PLAN]] | [[ARCHITECTURE]] | [[NETCODE]] | [[DETERMINISM]] | [[ROADMAP]]

---

## 1. Dead-code sweep (A + B)

Closes the loop on the big refactor. Two groups:

### A — safe, small (no caller anywhere)
- `src/boxes/pick_spawn_x.rs` (+ its `mod.rs` decl/re-export) — declared but never called.
- `src/boxes/spawn_x_for_player.rs` — the **boxes** copy is unused (the **engine's**
  `spawn_x_for_player` is the one that's used).
- `src/p2p/peer_id_to_player_id.rs` — `derive_player_id` no longer calls it.
- `src/boxes/spawn_box.rs` — confirm unused now that `render_snapshots` spawns bodies directly;
  remove if so.

### B — legacy snapshot protocol in `src/p2p/` (dead since the engine took over)
- `src/p2p/snapshot.rs` (`p2p::Snapshot`), `snapshot_codec.rs` (`SnapshotCodec`).
- The `positions` request-response protocol in `behaviour.rs` / `behaviour_new.rs`.
- `Command::SendSnapshot`, `Event::IncomingSnapshot`, the snapshot handling in `run.rs`, and the
  `two_swarm_snapshot_exchange` test.
- The snapshot-related constants / `PROTOCOL_NAME` (keep the netcode protocol constants).

**Rule:** verify `build` + `test` + `clippy` + `fmt` stay green after **each** removal.

---

## 2. Prediction / rollback — `bevy_lele_rollback_plugin_1`

**Decision (locked):** build **our own standalone, generic rollback crate** —
physics-agnostic, over a `Simulation` trait. Do **not** vendor `bevy_ggrs`; do **not** couple to
avian.

- **Crate name:** `bevy_lele_rollback_plugin_1` — a generic core with **zero bevy/avian deps**
  (plus an optional thin bevy link feature if we want one). It is reusable and future-proof.
- **Design:**
  ```rust
  trait Simulation {
      type State;                    // snapshot to save/restore
      type Input;                    // per-peer action for a tick
      fn step(&mut self, tick: u64, inputs: &[Input]);   // deterministic advance
      fn snapshot(&self) -> Self::State;                 // save a frame
      fn restore(&mut self, state: Self::State);         // roll back a frame
      fn hash(&self) -> u64;                             // canonical state hash (must match DETERMINISM)
  }
  struct RollbackSession<S: Simulation> { /* committed-frame buffer; predicted ahead state */ }
  ```
- **Behaviour:** the session keeps the authoritative committed frames (fed by
  `netcode::Lockstep`, commit-then-reveal). It predicts ahead a few ticks using **only local
  inputs**, and on each authoritative advance reconciles: if prediction diverged (a remote
  collision moved the local box), restore the last committed frame and re-simulate.
- **Integration (thin):**
  - `src/engine/` → `impl Simulation for Engine` (a small adapter; the engine is already pure
    `step` + snapshot).
  - render: **local** box from the **predicted** session; **remote** boxes from the **committed**
    (authoritative) frames.
  - `netcode::Lockstep` and the freenet membership + signed input log stay **unchanged** — this
    sits on top, not instead of them.
- **Research note (from the online check):** `bevy_ggrs 0.22` / `ggrs 0.13` is bevy-0.19
  compatible, but it is a full P2P session protocol that would replace our lockstep + freenet
  audit layer → rejected. `bevy_rollback` is bevy-0.5-era → unusable. Rolling our own over the
  `Simulation` trait is the smallest, architecture-correct fit, and lele-clean by construction.

---

## 3. Cross-OS / cross-network final verification (Windows on another network)

- **Windows build gate:** `build` + `test` + `clippy` + `fmt` all green on Windows.
- **Cross-OS determinism:** run the **same input trace** (the engine determinism gate) on Linux
  and Windows; assert the **final state hash is identical**. This proves avian `enhanced-determinism`
  holds across OS.
- **Two-machine, different-network live lockstep:** run 1–2 Linux + 1 Windows instance joining
  the **same contract** on the mainnet; wait for mutual convergence; record a screen video on each
  machine; send to Telegram (via `mainnet_automation_4` / the deskctrl+Telegram path).
- **Escalation:** if the cross-OS hash diverges, follow `DETERMINISM.md` → fixed-point fallback.

---

## 4. Commit-then-reveal strictness (optional)

- **Current:** commit + reveal sent together per tick; the reveal is hash-checked
  (`record_reveal` → tampering flagged) and the fixed buffer `D` closes same-tick reactions.
- **Stricter option:** wait for all peers' commits, then reveal (one extra round-trip).
- **Status:** accept the current simplification, or implement the full round-trip. *(decide)*

---

## 5. Remaining PLAN.md items (all optional / now covered)

| PLAN.md item | Where handled now |
|---|---|
| Prediction/rollback (render feel) | §2 — `bevy_lele_rollback_plugin_1` |
| Cross-OS determinism | §3 — Windows network gate |
| Strict commit-then-reveal | §4 (optional) |

---

## 6. Definition of FINISHED (final gate)

1. **Linux + Windows** all green: `build`, `test`, `clippy -D warnings`, `fmt`, `lele_lint`.
2. **Cross-OS determinism:** final state hash **identical** on Linux == Windows.
3. **Two-machine (different network) live lockstep** converges; **Telegram video** from each machine.
4. **Dead-code sweep (A+B) done** — no legacy snapshot remnants.
5. **`bevy_lele_rollback_plugin_1`** integrated + tested (prediction + rollback, determinism +
   reconciliation tests).
6. **Docs status updated** (PLAN/POLISH/ROADMAP) and the project declared **FINISHED**.