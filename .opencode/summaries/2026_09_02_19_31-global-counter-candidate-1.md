# global-counter-candidate-1 — 2026-09-02 19:31

- goal: harden GlobalCounterContract against client-forged MAX jump while keeping O(users) and running mainnet accumulation test with persistent logs
- why: forked WASM already isolated via ContractKey=Blake3(WASM), forked client on same key could send MAX via max-merge; strict +1 is minimal O(users) to block it without O(gap) buffer

- created: freenet_example/docs/candidates/CANDIDATE_1..4.md — 4 candidate decision docs
- created: freenet_example/src/testing/update_count_incrementally.rs:30 loop cur+1..=target via get_slot
- created: freenet_example/src/contract_wasm.rs:1 helper global_counter_wasm() with candidate_* features (stashed, now in worktrees)
- created: candidate branches/worktrees candidate/1-bounded..4-hashchain at ../freenet_example_candidate_* with isolated CARGO_TARGET_DIR /tmp/frt-build-candidate-N
- created: .opencode/summaries/ — repo-wide conversation summaries dir and LATEST.md pointer
- created: ~/.config/opencode/commands/save-conversation.md — repo-wide /save-conversation command

- edited: freenet_example/contract/Cargo.toml:2 global_counter_contract, contract/src/lib.rs:7 GlobalCounterContract struct+tests
- edited: freenet_example/build.rs:46 wasm_target_dir="contract/target" isolated from CARGO_TARGET_DIR to avoid Blocking waiting for file lock self-deadlock
- edited: freenet_example/devenv.nix:30-40 tasks to {exec,showOutput=true} and 25 isolated target, skill devenv-rs:238 ban | tail
- edited: freenet_example/src/testing/build_game.rs:17 bin freenet-example-3 -> freenet-example
- edited: freenet_example_candidate_1/freenet_example/contract/src/lib.rs:48 bounded if value > cur+1 continue with harness_candidate_1 and bounded_increment tests
- edited: freenet_example_candidate_1/freenet_example/tests/e2e_local/mainnet.rs:22 parse_last_count strip_ansi, 65 persistent .local-run/<timestamp>-<prefix>/, 120 accumulation 30s/10% gap with per_instance+1 slack
- edited: freenet_example/src/lib.rs:10 GlobalCounterClient, global_counter_error, contract_wasm pub use
- edited: 8 integration tests (connect.rs:111, full_lifecycle.rs:12, persistence.rs:13, publish_subscribe.rs:10, state.rs:46, multi_subscriber:17, two_writers:16, example publish_subscribe) to use update_count_incrementally and drain notifications
- edited: ~/.config/opencode/skills/devenv-rs/SKILL.md:238 WASM isolation + showOutput + tail ban

- verified: freenet_example_candidate_1 freenet_example contract 3/3 cargo test (harness_candidate_1, bounded_increment)
- verified: freenet_example_candidate_1 devenv tasks run lele:nextest 54 passed (1 slow gateway_subprocess_smoke)
- verified: freenet_example_candidate_1 mainnet_local 5+5 runs PASS (210s,102s,87s,102s,101s then 123s,132s,137s,82s,131s with accumulation 30s/10% after strip_ansi fix)
- verified: freenet_example devenv tasks run lele:clippy/lint/taxonomy, freenet:contract-harness 13 passed

- decisions: keep candidate/1-bounded strict +1 (breaking, needs incremental helper) vs window N=5+sig minimal O(users) for candidate/2-signed; buffer O(gap) 10MB rejected; do nothing leaves MAX vuln
- decisions: branch/worktree model (candidate/*) over shared-host contract/candidates/ for parallel client divergence; per-worktree /tmp/frt-build-candidate-N isolated

- next: implement Window N=5 + Signatures in candidate/2-signed (Parameters allow-list, tick.rs ed25519 sign, update_state verify then cur+5 window) and rerun 5× mainnet with same accumulation test
