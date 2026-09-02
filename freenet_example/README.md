# Freenet Global Counter Example

A shared counter that runs across the Freenet P2P network. The binary starts its
own Freenet node, joins mainnet, and increments a shared counter every second.
Multiple machines that start at similar moments converge on one counter — no
server, no configuration, no IP sharing.

## Quick Start

Download the latest binary for your OS from the repo's GitHub Releases, then:

```bash
chmod +x freenet-example-linux
./freenet-example-linux
```

Development build (build.rs compiles the contract WASM automatically):

```bash
rustup target add wasm32-unknown-unknown   # once
cargo build --release
cargo run --release
```

Testing tiers:

| Command | What | Internet? |
|---------|------|:---:|
| `cargo test --all-targets` | contract + library + integration tests | No |
| `cargo run --manifest-path e2e_mainnet/Cargo.toml --release -- 3 --repeat 5 --mode counter` | live mainnet harness: 3 instances × 5 trials | Yes |
| `cargo test --test cross_os_reconcile -- --ignored --nocapture` | cross-OS mainnet probe (CI-driven, needs env vars) | Yes |

Static checks: `cargo clippy --all-targets -- -D warnings`, `cargo fmt -- --check`,
`cargo run --manifest-path ../lele_lint/Cargo.toml`.

---

# Code Map for a Future Agent

**The project is feature-complete as a reference implementation.** This section
tells you what is load-bearing, what you may delete, and where the hard-won
knowledge lives.

## Philosophy for changes here

- Code size is not line count. 100 useful functions are fine. What matters:
  **one function does one thing; structs hold data; anything unused is deleted.**
- Method files (`<type>_<method>.rs`) are private modules reached only through
  thin delegates on the struct. Structs hold fields only. `lele_lint` enforces
  this — always run it after changes.
- This crate pins `freenet =0.2.101` / `freenet-stdlib =0.8.3`. The behavioral
  notes below are verified against exactly those sources.

## ESSENTIAL — do not remove, the app stops working without it

| Part | Why it exists |
|---|---|
| `contract/src/lib.rs` | The whole contract: `Slots = BTreeMap<u64, u64>` CRDT. `update_state` max-merges per tag (the ONLY merge rule), `summarize_state` returns the per-tag map (not a total — equal totals can mask divergence), `get_state_delta` returns only lagging tags. The committed `contract/global_counter_contract.wasm` pins the contract key: **never rebuild per deployment** (rebuild ⇒ new code hash ⇒ new key ⇒ different "room"). |
| `src/global_counter_client.rs` + `src/global_counter_client_method/` | The counter client. `connect.rs` does Get → Put-only-on-NotFound (putting unconditionally overwrites network state). `tick.rs` absorbs notifications (max-merge, never replace), increments own tag, sends single-tag `UpdateData::State`. `bridge_tick.rs` is split recovery: `ContractRequest::Subscribe { key, summary }` every 30s while no foreign activity, then an idempotent re-Put. `note_foreign_slots.rs` tracks foreign freshness (see invariants). `merge_slots.rs`, `foreign_tags.rs`, `count/own/state/contract_key` support these. |
| `src/freenet_client.rs` (+ methods), `src/recv_after_get.rs`, `src/recv_response.rs` | Minimal WebSocket client for `ws://127.0.0.1:<port>/v1/contract/command?encodingProtocol=native`. `recv_after_get`/`recv_response` encode the response-loop discipline: messages arrive FIFO with no request correlation — loop and skip `SubscribeResponse`/`UpdateNotification`. |
| `src/main.rs` | Node bootstrap (in-process network-mode node, mainnet client) + 1s tick loop calling `tick → note_foreign_slots → bridge_tick`. |
| `build.rs` | Builds + commits `contract/*.wasm` (mtime-checked). |
| `integration_tests/` | The only behavioral gates that run offline. `cross_os_reconcile.rs` is `#[ignore]`d and driven by CI (`.github/workflows/self-hosted-ci.yml` → `cross-os-reconcile` jobs). |
| `e2e_mainnet/` | The live mainnet harness that produced all verified results. Entry: `parse_config.rs` → `trial_run_trial.rs`; verdict math in `reconcile_analyze.rs` (absolute tolerance `#instances + 3`, tick-only parsing), convergence detector counts consecutive update observations backed by advancing tick generations. |

## Essential invariants (learned the hard way, on mainnet)

1. **Absorb notifications by max-merge, never replace.** Ticks send single-tag
   updates; replace-based absorption wipes foreign slots from the client view
   every tick and the app looks permanently unmerged.
2. **Get from a hosting node answers locally** — re-Get/re-Get-subscribe cannot
   bridge splits. Re-Put relays via the gateway and times out (90s) without
   bridging. Only the routed `Subscribe` op bridges; heals ~10–60s vs ~300s
   (the node's 5-min interest heartbeat).
3. **Arm the bridge on two conditions**: no foreign slot ever seen, AND foreign
   values no longer advancing (frozen subscription — node logs
   `BROADCAST_NO_TARGETS`). Freshness = foreign **value sum** changed, not
   foreign-slot presence.
4. **Concurrent fresh-key Puts can seed disjoint replica groups** (core has no
   ring-wide reconciliation; anti-entropy is neighbor-pair-only, 300s cadence).
   This is the network's behavior, not a contract bug — the contract's max-merge
   already reconciles once any bridge succeeds.
5. **Idempotency**: every update carries its monotonic value (`{tag, own}`);
   `update_state` reads new values from `data`, never increments from `state`.
6. **Ring membership is a prerequisite, and VPN/datacenter NAT breaks it.**
   Behind a ProtonVPN exit the node reached gateways but never joined the ring
   (`ring_connections=0`, `RING_TRANSPORT_DESYNC`, peer dials fail NAT
   traversal): its state still propagated *out* via gateway relays, but it
   could not receive anything — reconcile impossible regardless of bridging.
   Run nodes on NATs that permit hole-punching.

## TRIMMABLE — candidates if you want a smaller codebase

| Part | Why it's expendable |
|---|---|
| `src/set_client.rs` + `src/set_client_method/` + `contract/src/set_contract/` + `contract/set_contract.wasm` + the `Set` arm in `main.rs`/e2e | The "set" contract mode is a second, parallel implementation used by `--contract-mode set`, which the harness no longer exercises by default and which never got the bridge/recovery work. Removing it deletes ~2 crates' worth of code and the `SetClient` branches in `main.rs` + `e2e_mainnet`. Decide first whether a set/OR-set demo is still a goal; if not, delete wholesale. |
| `unruled_examples/` | Contains only its README; the exemption it describes has no current content. |
| `scenarios/` + `examples/` (partially) | Keep `standalone_demo` and `publish_subscribe` as the minimal teaching examples. `p2p_counter_ws_bridge` (119 lines) demonstrates a manual WebSocket bridge no longer needed since the binary is standalone; `p2p_counter_gateway`/`connect_to_external` overlap with `--role`-based external-node usage. Candidates for deletion if docs are updated accordingly. |
| `src/role.rs` (`Role::Subscribe`) | `main.rs` always uses `Role::Publish` (publishers Get-first and recover via bridge). `Role::Subscribe` exists only for `integration_tests/publish_subscribe.rs`; folding that test onto `Role::Publish` would let the whole enum go. Low priority. |
| `Makefile.toml` tasks | Largely superseded by plain cargo commands + the e2e harness; several tasks reference flows that changed. |
| `docs/` claims in old `README` (pre-rewrite) | Referenced a removed `e2e_tests/` crate; fixed in this rewrite. |

## IMPROVABLE — works, but would be cleaner

1. **`src/testing/` module**: exists to support unit tests via a `TestNode`
   helper. Fine as-is, but if the set-contract goes away, its set-related
   helpers go too.
2. **`e2e_mainnet` GUI coupling**: the harness drives real `xterm` windows and
   screen recording (launch/tile/record). Reliable, but Linux-only and
   screen-dependent. A headless spawn mode would make it CI-runnable without a
   desktop.
3. **`bridge_tick.rs` leg attribution**: the harness counts `bridge: split
   suspected` accurately but merge attribution is fuzzy (heals are usually
   observed via the next absorbed notification, not by the leg that caused
   them). If you need per-leg success rates, emit an explicit post-check.
4. **`Role` enum**: see TRIMMABLE above — one variant doing all the work.
5. **`e2e_mainnet/src/outcome.rs` vs `reconcile_result.rs`**: two result
   aggregates with overlapping fields (instance-level vs trial-level). Could be
   one layered type.

## Where the deeper knowledge lives

- `freenet-gateway` skill: fresh-key Put race root cause, anti-entropy
  neighbor-pair semantics, node roles/bootstrap flags.
- `freenet` skill: client patterns, replica-split bridging, notification
  absorb-merge.
- `freenet-contract-design` skill: CRDT/monoid design, per-tag summaries,
  trust/anti-cheat tradeoffs.
- `.local-run/` (gitignored): raw instance logs from every mainnet trial —
  the evidence base for all invariants above.
