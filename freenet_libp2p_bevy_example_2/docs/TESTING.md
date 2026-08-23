# TESTING

How `example_2` will be tested once implementation (Phase 3) starts. This is both a **runbook**
(proven patterns to iterate with) and a **blueprint** (walk from example_1's testing toolkit).

[[README]] | [[FINDINGS]] | [[ARGUMENT]] | [[DESIGN]] | [[SKILL_PLAN]]

---

## 1. Purpose & iteration loop

`example_1` invested heavily in testing the app "properly", including a dedicated `testing`
crate. This file captures that investment so `example_2` iterates fast instead of re-inventing
the harnesses.

The design leans toward **authority-in-contract** ([[ARGUMENT]] Framing A): membership, schema,
and authorization live in the wasm contract. That makes the **contract test suite the critical
gate** — most of the correctness that protects "same-app peers connect" is enforced there, so
we test the contract first and hardest.

Per-change loop (mirrors example_1's Makefile `pre-push`):

```bash
# build the contract wasm first, then everything else
CARGO_TARGET_DIR=/tmp/frt-build cargo build --workspace --all-targets
CARGO_TARGET_DIR=/tmp/frt-build cargo test --workspace --all-targets
cargo run --manifest-path ../lele_lint/Cargo.toml
CARGO_TARGET_DIR=/tmp/frt-build cargo clippy --workspace --all-targets -- -D warnings
CARGO_TARGET_DIR=/tmp/frt-build cargo fmt -- --check
```

- `CARGO_TARGET_DIR=/tmp/frt-build` is required because `freenet` → `tikv-jemalloc-sys` fails
  to build when the source path contains spaces (the workspace root is under
  `Syncthing/[AAI] Agentic AI/...`).
- `lele_lint` enforces the per-file `test_usage` convention (see [[SKILL_PLAN]] and the
  `lele-lint-rs` skill).

---

## 2. The example_1 toolkit (what we carry over)

example_1 structured testing around a **dedicated `testing` crate** plus **per-layer test
crates**. The key assets:

### 2.1 Contract tests (in `contract/src/lib.rs`)

The roster contract ships a full merge-law suite that must be mirrored (and extended) when
example_2 hardens its contract:

- `test_update_state_is_commutative` — merge order does not matter.
- `test_update_state_is_idempotent` — same update twice → same result.
- `test_update_state_is_associative` — grouping does not matter.
- `test_get_state_delta_carries_the_roster` — regression guard: delta must not collapse to
  empty or peers stop syncing.
- `test_invalid_state_rejected` — `validate_state` rejects malformed bytes.

These are pure, fast, deterministic — run on every change with `cargo test`.

### 2.2 Shared harness: the `testing` crate

Structs:

| Struct | What it exercises | Runs how |
|--------|-------------------|----------|
| `TestNode` | An embedded freenet node harness; `start_gateway(port)` / `start_peer(gateway_port, pubkey)` wire nodes directly (hermetic, no mainnet). | Port verbatim — freenet-only, app-agnostic. |
| `TestGameApp` | A hermetic Bevy `App` talking to a WS test node; covers roster/boxes logic but **not** the real production node-discovery path. | Re-create against example_2 app types. |
| `ProductionGameApp` | The **full production startup path** (`load_or_create_keypair` → `p2p::run` → `start_embedded_node` → `connect_client_loop`) inside a real `App`. | Re-create against example_2 app types. |

Methods:

| Method | Purpose |
|--------|---------|
| `unique_params()` | Fresh, run-unique contract `parameters` per call → each run deploys to a new key, so stale mainnet entries from a prior run can't pollute assertions. |
| `load_wasm()` | `include_bytes!` the contract wasm into the test binary. |
| `connect(host, port)` | WebSocket client connect to a node. |
| `deploy_roster(...)` | Put + subscribe a fresh roster contract, returns client + initial roster view. |
| `recv_roster_notification(...)` | Loop-skip the WS channel for roster `UpdateNotification`s. |
| `wait_for_roster_len(client, n, timeout)` | Poll until the roster reaches `n` entries. |
| `check_internet_access()` | Guard for mainnet-dependent tests. |

### 2.3 Layer → coverage → invocation map

| Crate/test | Covers | Run with |
|------------|--------|----------|
| `contract/src/lib.rs` | Merge laws, delta integrity, validate rejection | `cargo test -p roster_contract` |
| cargo unit tests (`test_usage`) | Per-file logic, no Bevy/node | `cargo test` |
| `integration_tests/two_node_roster.rs` | Two directly-wired nodes converge on the roster (M2 checkpoint) | `cargo test --workspace` |
| `integration_tests/two_node_box_count*.rs` | Two-node physics/box sync (hermetic) | `cargo test --workspace` |
| `integration_tests/local_two_node_production_sync.rs` | **Deterministic gate**: two production-path nodes directly wired converge + sync movement | `cargo test --workspace` |
| `e2e_tests/e2e_three_node_production_sync.rs` | Real production path over public mainnet, staggered join | `cargo test --test e2e_three_node_production_sync -- --ignored` (`#[ignore]`) |
| `cross_os_tests/*` | Per-machine roster presence / movement, JSON-lines logs, verified by a CI job | `#[ignore]`, run per machine |
| `firewall_probe/` | freenet-only NAT/connectivity probe | separate crate |
| `mainnet_automation/` | Desktop automation: spawn real app windows, drive with keys, record video, Telegram | separate bin |

---

## 3. Core repeatable patterns

1. **Contract = the correctness gate.** Encode invariants (schema, caps, LWW monotonicity,
   signed writes) in the contract; test them there. See [[ARGUMENT]], [[DESIGN]].
2. **`unique_params()` every run.** Roster/contract keys persist on the network; always deploy
   to a fresh params-derived key in tests so a previous run's state can't leak in.
3. **`load_wasm()` via `include_bytes!`.** Embed the built contract wasm so the test binary is
   self-contained.
4. **Hermetic before production.** Prove logic against directly-wired `TestNode`s first
   (`local_two_node_production_sync` is the deterministic gate); use the public mainnet only in
   `#[ignore]`d e2e tests.
5. **`#[tokio::test(flavor = "multi_thread")]`.** wasmtime uses `spawn_blocking`, so async tests
   need the multi-thread runtime.
6. **`#[ignore]` for mainnet/OS-dependent tests.** Keep the default `cargo test` green and fast;
   gate external-dependency tests behind `--ignored`.
7. **`CARGO_TARGET_DIR=/tmp/frt-build`.** Mandatory in this workspace (space-in-path).
8. **Makefile tasks.** Mirror `build-contract`, `copy-wasm`, `test`, `pre-push`, `clean`.

---

## 4. Example_2 implementation blueprint

Ramp coverage as the app is built; the contract comes first.

### Phase 3a — Contract (hardened, authority-in-contract)
- [ ] Author the membership contract per the chosen framing ([[DESIGN]]).
- [ ] Replicate example_1's merge-law suite **plus** the new invariants ([[DESIGN]] rules):
      - Reject a write to an existing member that is **not signed by that member's stored
        `identity_key`** (anti-impersonation).
      - Reject a **rewind** (`new.seq ≤ stored.seq`) for a member's own entry.
      - Reject a **new member** whose entry is **not self-signed** by its `identity_key`.
      - Reject **over-`max_members`** and **over-`MAX_ADDRS`** in `validate_state`.
      - Keep merge laws lawful: commutative / associative / idempotent per-entry LWW on `seq`.
- [ ] Build-pin check: assert the committed canonical `.wasm` bytes are used so all clients
      share one contract (**a rebuild changes `ContractKey`** — [[DESIGN]] canonical-`.wasm`
      note; also the `freenet` skill).
- [ ] Deterministic `cargo test -p <contract>` green before any client work.

> **Out of scope (this crate):** input-truth anti-cheat. The contract guarantees identical
> *functions*, not *honest inputs*; anti-cheat testing (deterministic re-sim / referee) is
> future work elsewhere, so no anti-cheat test cases are designed here.

### Phase 3b — Client scaffold + local harness
- [ ] Port `cli/`, `freenet/` (WS client) verbatim from example_1.
- [ ] Re-create the `testing` crate for example_2: `TestNode` pattern verbatim; re-create
      `TestGameApp`/`ProductionGameApp` against example_2's `NetworkId`/roster types
      (see [[#5-adaptation-notes]] §5).
- [ ] `integration_tests/` equivalents: two-node roster converge + the deterministic
      production-path gate.

### Phase 3c — Real-time p2p + game module
- [ ] Re-create `p2p/` transport tests and the box/physics game module.
- [ ] Two-node movement-sync integration test (mirror `two_node_box_count` /
      `local_two_node_production_sync`).

### Phase 3d — External-path tests (deferred until core is green)
- [ ] `e2e_tests/` (mainnet, `#[ignore]`), auth/signed-write negative tests.
- [ ] `cross_os_tests/` and `firewall_probe/` if multi-machine is needed.
- [ ] CI wiring via `crate-tag-ci` / self-hosted pipeline / `test-orchestrator`.

---

## 5. Adaptation notes (brief)

- **Port verbatim (app-agnostic):** `TestNode`, `unique_params`, `load_wasm` pattern, the
  commutetive-merge contract-test patterns, the Makefile skeleton, `CARGO_TARGET_DIR` usage.
- **Must re-create (reference example_1 app types):** `TestGameApp` / `ProductionGameApp`, the
  integration/e2e test bodies, and any methods that touch example_1's `boxes::PlayerId` /
  `roster::PeerEntry` — example_2 has its own types (`NetworkId`, hardened `PeerEntry`).
- The detailed adaptation map is left to Phase 3 implementation planning, not this doc.

---

## 6. CI & automation (mapped; deferred)

Inventory for when example_2 grows into external verification. Wired later, not during code
iteration.

| Layer | Purpose | Status |
|-------|---------|--------|
| `self-hosted-ci.yml` + `test-orchestrator` | Self-hosted test gate + Linux/Windows builds on own machines | Deferred |
| `crate-tag-ci` | Tag-triggered test / build-check / release (`<crate>-(test\|build\|release)-...`) | Deferred |
| `network-probe.yml` | Detect same-LAN runs (public + LAN IP per machine) | Deferred |
| `firewall-probe.yml` | NAT/connectivity probe | Deferred |
| `mainnet_automation/` | Real desktop end-to-end: spawn windows, drive keys, record video → Telegram | Deferred |

See the local `crate-tag-ci` skill and the `test-orchestrator` MCP for how these run today on
example_1.