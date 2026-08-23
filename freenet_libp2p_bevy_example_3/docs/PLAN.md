# PLAN

Complete handoff plan to build the **final example_3 app**: deterministic-lockstep netcode with
avian (enhanced-determinism) physics, hybrid libp2p + freenet transport, reusing the example_2
test stack. This is a single coherent plan (no milestone phases) that produces a working, tested
app.

[[README]] | [[DIFFERENTIATION]] | [[ARCHITECTURE]] | [[CONTRACT]] | [[NETCODE]] | [[DETERMINISM]] | [[ANTI_CHEAT]] | [[ROADMAP]]

---

## 1. Goal

Build `freenet_libp2p_bevy_example_3`: a peer-to-peer game where every client runs the **same
deterministic physics**, nobody sends a position (only inputs + state hashes flow), and cheating
reduces to *choosing inputs* (accepted). Members are identity-keyed and self-certifying; the
freenet contract keeps membership + a signed input log; libp2p carries the real-time lockstep.

This differs from example_2: positions are **not** client-computed snapshots — they come from the
shared engine ([[DIFFERENTIATION]]).

## 2. Architecture (summary + docs)

- **Engine (authority):** avian2d made deterministic (below). Sole producer of positions.
- **Live path:** libp2p — commit-hash → reveal-input → state-hash, per tick, under **fixed
  command delay + commit-then-reveal** ([[NETCODE]]).
- **Enforced layer:** freenet contract — identity-keyed membership + a **signed input log**
  (state hashes + bounded recent-input ring) for audit/rejoin ([[CONTRACT]]).
- **Render:** Bevy draws engine snapshots; client-side prediction + rollback ([[ARCHITECTURE]]).

## 3. Decisions (locked)

| # | Decision | Choice |
|---|---|---|
| Engine | avian2d `=0.7.0`, `enhanced-determinism`, `parallel` **off**, fixed 60 Hz | |
| Contract commits | state-hashes + **bounded recent-input ring** (not every tick) | |
| Log writer | each member appends its **own signed** contributions | |
| Tick rate | 60 Hz | |
| Liveness `B` | peer offline > **5 s (300 ticks)** is excluded; rejoin from log | |
| Render feel | **client-side prediction + rollback** | |
| Determinism verify | **local** now; cross-OS deferred (escalation recorded) | |
| Test harness | mirror example_2 → `testing_3`, `integration_tests_3`, `e2e_tests_3`, `mainnet_automation_3` | |
| Names | crate `bevy_freenet_3`, lib `freenet_libp2p_bevy_example_3_lib`, bin `freenet-libp2p-bevy-example-3`, own `contract` | |
| Defaults | buffer `D = 4` ticks (~67 ms); `B = 300` ticks (5 s); avian deterministic schedule | |

## 4. Engine — avian made deterministic (the core nuance)

- **Headless sim owns physics.** One isolated avian world stepped by a single-threaded,
  deterministic `FixedUpdate` schedule (the pattern in avian's own determinism test: no rendering
  plugins, fixed order). `advance(state, inputs) -> state`.
- **Render app only draws.** The Bevy display app reads engine snapshot output and interpolates /
  rolls back; it must **not** register a physics plugin (a second sim would diverge).
- **Deterministic state hash.** Serialize the sim snapshot **canonically** (stable field order,
  `BTreeMap`, no `HashMap`) and hash it — this is what peers compare every tick.
- **Config:** enable `enhanced-determinism`, disable `parallel`, fixed timestep 60 Hz.
- **Honest caveat:** avian determinism is *improved* (f32 + IEEE-754 compliance), not a
  guarantee. A cross-OS bit-divergence → state-hash mismatch → detected/excluded. Escalation:
  enable the deferred cross-OS determinism run, or fall back to a hand-rolled fixed-point engine
  (documented in [[DETERMINISM]]).

## 5. File / module layout (lele-atomic, `_3` names)

```
freenet_libp2p_bevy_example_3/
  Cargo.toml                 # workspace: ., testing_3, integration_tests_3, e2e_tests_3, mainnet_automation_3 ; exclude contract
  build.rs                   # build+embed contract/membership_contract_3.wasm
  .cargo/config.toml         # CARGO_TARGET_DIR + mold (mirror example_2)
  Makefile.toml
  src/
    lib.rs                   # pub mod cli/freenet/p2p/roster/boxes/render/engine
    main.rs                  # cli→keypair→p2p::run→sign entry→connect_and_run→bevy
    cli/                     # port example_2 (identity-dir, params, local, gateway)
    freenet/                 # port example_2 WS client
    boxes/                   # render-only boxes over engine snapshots (drop avian physics here)
    engine/                  # avian headless sim: advance(state, inputs)->state; canonical hash
      mod.rs
      fixed.rs               # (present for the fallback) — kept for the documented fallback
      sim.rs                 # headless avian world + DeterministicSchedule step
      snapshot.rs            # BodySnapshot { id, pos, vel }; canonical serialize+hash
      hash.rs                # canonical state hash (BTreeMap, stable order)
    p2p/                     # port example_2 swarm; add per-tick commit/reveal/state-hash messages
    roster/                  # membership (example_2 pattern) + signed input log append
    netcode/                 # fixed command delay D, liveness B, commit-then-reveal, Option A sort
    render/                  # bevy display: interpolate snapshots, prediction+rollback
  contract/src/              # membership + signed input log (new code => new key/room)
  testing_3/ integration_tests_3/ e2e_tests_3/ mainnet_automation_3/
```

Follow lele atomic-file conventions (one pub item per file, `test_usage`, module imports,
thiserror, no inline `crate::` outside `use`), exactly as example_1/2.

## 6. Build steps (sequential tasks that together produce the final app)

1. **Scaffold** the workspace (Cargo.toml, build.rs, .cargo/config.toml, Makefile.toml), mirror
   example_2 structure with `_3` names. Deps: bevy `=0.19.0`, avian2d `=0.7.0`
   (`enhanced-determinism`, `parallel` off), freenet `=0.2.128`, freenet-stdlib `=0.8.5` net,
   libp2p `=0.56.0`, tokio, tokio-tungstenite, bincode, serde, thiserror, tracing, clap, etc.
   (copy example_2's list, edit avian features).
2. **Engine** — headless avian sim with a deterministic schedule; `advance(state, inputs)`;
   body box/gravity/jump/grounded; canonical snapshot serialize + hash. Include the
   `test_usage` determinism test (same input trace twice → identical hash).
3. **Boxes render** — move example_2 `boxes` to read engine snapshots (interpolation), dropping
   avian-from-the-render path so only the headless sim owns physics.
4. **Netcode** — port example_2 `p2p`; add the tick protocol: commit `hash(input_N)` → reveal
   `input_N` → Option A ordered `advance` → broadcast `hash(state_{N+1})` → compare;
   fixed command delay `D = 4`; liveness `B = 300`; missing-input timeout.
5. **Contract** — membership (example_2 pattern, identity-keyed, self-certifying, monotone `seq`,
   caps) + signed input log: per-member `log_seq` + bounded ring of `HashedInput`; validates
   form/auth/monotonicity and caps; merge commutative.
6. **Wire `main.rs`** — cli → keypair → p2p::run → build + sign own entry → connect_and_run;
   Bevy render over engine snapshots with prediction + rollback.
7. **Render feel** — client-side prediction on own input; rollback on tick commit; interpolation.
8. **Session/liveness** — join via membership; rejoin from contract signed log; exclude offline
   > `B`; resume.

## 7. Testing (reuse the example_2 stack)

- **Adapt** the example_2 harness to the new types:
  - `testing_3` `TestNode` (freenet gateway/peer) + `ProductionGameApp` (full startup path);
    `unique_params` → new `Params`; deploy/sign entries.
  - `integration_tests_3`: two-node roster converge; `local_two_node_production_sync_3`.
  - `e2e_tests_3`: mainnet `#[ignore]`.
  - `mainnet_automation_3`: drive N real instances + Telegram video.
- **Add**, per the plan:
  - **Determinism gate** (local): run a fixed input trace twice (in-process + across two temp
    processes) → assert identical state hash. Cross-OS run is deferred (escalation; see §4).
  - **Lockstep convergence**: two nodes reach the same `state` hash after K ticks.
  - **Fairness**: a peer beyond `D`/`B` is idle/excluded, never favored; commit-then-reveal blocks
    same-tick reactions (tampered reveal flagged).
  - **Contract**: merge-law suite + negatives (unsigned input, rewind, over-cap) — mirror
    example_2 `contract/src`.
- Every `src/` module keeps `test_usage`.

## 8. Verification (recurring)

```bash
CARGO_TARGET_DIR=/tmp/frt-build cargo build --workspace --all-targets
CARGO_TARGET_DIR=/tmp/frt-build cargo test --workspace --all-targets
CARGO_TARGET_DIR=/tmp/frt-build cargo clippy --workspace --all-targets -- -D warnings
cargo fmt -- --check
# lele per crate (default scans that crate's src):
cargo run --manifest-path ../lele_lint/Cargo.toml                      # from the example_3 root
(cd contract && cargo run --manifest-path ../../lele_lint/Cargo.toml)  # from the contract crate
# visual local-mainnet run (mirror example_2):
CARGO_TARGET_DIR=/tmp/frt-build cargo run -p mainnet-automation-3 -- 3
```

## 9. Tunable constants (where they live)

- `D` (command buffer) = `4` ticks — `src/netcode/constants.rs`.
- `B` (liveness) = `300` ticks (5 s at 60 Hz) — `src/netcode/constants.rs`.
- Tick rate `60` Hz — engine schedule + `D`/`B` are expressed in ticks.
- avian `enhanced-determinism` on / `parallel` off — root `Cargo.toml`.
- Input-log ring size + per-tick rate cap — `src/roster/` / `contract` constants.

---

Prepared so another agent can implement the final app end-to-end. Docs are the authority; the
design details are in the linked files above.