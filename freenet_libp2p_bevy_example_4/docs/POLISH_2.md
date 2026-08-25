# POLISH_2 — Path to CONCLUDED & POLISHED

Supersedes `docs/POLISH.md`. Reflects three decisions from review:

- **Live-join catch-up:** IN SCOPE (a mid-session joiner must converge to identical state hashes).
- **Anti-cheat:** detection-only for now (input-lying is accepted per DIFFERENTIATION); the signed
  `input_log` + `tampered` exclusion stay built + unit-tested but are not wired into the live path.
- **Final gate:** FULL cross-OS gate in scope (Windows build + cross-OS determinism + two-machine
  live lockstep).

The convergence root cause (libp2p `request_response` misuse — never answering inbound requests)
is **already fixed**; see `docs/CONVERGENCE_INVESTIGATION.md`.

---

## Definition of CONCLUDED (the gate)

1. Linux green: `build`, `test`, `clippy -D warnings`, `fmt`, `lele_lint`.
2. **3+ consecutive** `mainnet_automation_4` end-to-end passes (no flakes).
3. **Windows green** (build / test / clippy / fmt / lele) via crate-tag CI.
4. **Cross-OS determinism:** `engine_determinism` un-`#[ignore]`d, run on Linux **and** Windows;
   final state hash **identical**.
5. **Two-machine, different-network live lockstep:** 1–2 Linux + 1 Windows instance join the same
   contract, mutually converge, per-machine video recorded.
6. **Live-join catch-up** implemented + tested (late joiner converges hashes).
7. **Docs reconciled** (README, POLISH, DETERMINISM, ANTI_CHEAT); project declared CONCLUDED.

---

## Phase A — Reliable convergence (fix the two confirmed flakes)

### A1. Fresh-key deploy race
`mainnet_automation_4/src/new_contract_params.rs`, `src/roster/setup_contract.rs`

- **Preferred:** **stable contract key** — the automation reuses a persistent `--contract-params`
  key per namespace, so the contract pre-exists on mainnet and there is no `Put` race (no
  `Get`→`NotFound` grace-window → `timeout after 30s` → infinite retry stall).
- **Fallback:** make `setup_contract` never strand — on `Get` timeout / `NotFound` after the grace
  window expires, `Put` gracefully instead of retrying setup forever.
- **Verify:** 3+ consecutive runs each show a clean `1 Put` seed and start the roster loop.

### A2. TTL prune un-convergence
`src/roster/constants.rs`, `src/roster/prune_stale.rs`, `src/roster/connect_client_loop.rs`

- Don't prune a peer whose Freenet refresh delivery stalled but that is still alive. Options:
  lengthen `ROSTER_ENTRY_TTL_SECS`, base pruning on last-seen-live, or suppress pruning within a
  run's convergence window.
- **Verify:** a long run never shrinks a full 3/3 roster.

## Phase B — Anti-cheat (detection-only, by decision)

- **No code wiring.** Keep state-hash divergence as detect+log
  (`src/boxes/bevy_systems/netcode_tick.rs:131-156`); do **not** submit client inputs to the
  contract `input_log` and do not implement wire-level exclusion now.
- **Docs:** add an explicit note in `docs/ANTI_CHEAT.md` / `docs/DIFFERENTIATION.md`: enforcement is
  detection-only; input-lying is accepted; the contract's signed `input_log` + `tampered` exclusion
  remain available and unit-tested but are not wired into the live path. (Wiring them is optional
  future work.)

## Phase C — Live-join catch-up (in scope)

Implement the deferred PLAN Phase 3 (re-baseline snapshot):

- **Messages** in `src/p2p/netcode_msg.rs`: `RequestSnapshot {}` and
  `Snapshot { tick, bodies: BTreeMap<PlayerId,(x,y,vx,vy)>, participants, from }` — full
  `EngineSimState` (including velocity), NOT the positions-only `Snapshot`.
- **Handshake:** on `PeerConnected` to an established peer, joiner C sends `RequestSnapshot`; each
  established peer replies its authoritative `Snapshot`; C picks the deterministic authority
  (lowest `PlayerId.from`) and cross-checks its state hash against a second peer before adopting.
- **Re-baseline:** at snapshot tick T all peers deterministically spawn C's body at default
  position; C `restore()`s the authoritative state and seeds its own body; all set
  `Lockstep::applied_through = T` and participants = snapshot set (+C); continue forward so hashes
  reconverge from T. Live join preserved (no reset to tick 0).
- **Tests:** `integration_tests_4` — two peers converge, a third joins mid-session, assert all
  three converge on identical state hashes. Interacts with A1 (joiner must share the same stable
  contract key).

## Phase D — Cleanup + docs

- Remove inert `netcode::simulate_lockstep` (no live caller); keep `engine::run_trace` behind the
  cross-OS determinism gate (it powers that gate).
- Fix **`COMMAND_DELAY`** doc mismatch (code = 8, PLAN = 4).
- Rewrite **`README.md`** — it is a working app, not "Docs only. No code."
- **Reconcile `POLISH.md`:** drop the stale §1 (example_2-only files; `spawn_box.rs` is actually
  used), stop framing §2 (rollback) as future (it is done), and fold §3/§6 into this gate.

## Phase E — Verification gate

- Linux full green + 3+ consecutive `mainnet_automation_4` E2E passes.
- **Windows gate** via crate-tag CI (build/test/release) pulling the Windows binary
  (`CARGO_TARGET_DIR=/tmp/frt-build` honored for the space-in-path workspace).
- **Cross-OS determinism:** run `cross_os_tests_4::engine_determinism` on Linux + Windows; assert
  equal final hash.
- **Two-machine live lockstep:** self-hosted pipeline + `cross_os_tests_4` / `launch_game` on a
  Linux + Windows pair joining the same contract; record per-machine video; document results.
- **Declare CONCLUDED** and update status docs.

---

## Effort / risk

| Item | Effort | Risk |
|------|--------|------|
| A1 fresh-key deploy | small | low |
| A2 TTL prune | small | low |
| B anti-cheat docs | trivial | none |
| C live-join catch-up | largest | medium-high (dynamic-membership lockstep) |
| D cleanup/docs | small | low |
| E verification gate | medium | partly infra (CI / self-hosted) |

Build order: **A1 → A2 → D → B**, then **C**, then **E**. Do not run the full gate until A and C
are green.