---
description: "Run the full local cross-machine implementation on the two self-hosted runners via the test-orchestrator MCP. Default: fast dev build + tests on Linux AND Windows + cross-OS mainnet sync. Optional 'release' argument switches to a release build. Stops and asks before proceeding if the two machines share a LAN."
---

You are orchestrating the local cross-machine pipeline for the freenet-libp2p-bevy game crate via the `test-orchestrator` MCP. Everything runs on the two self-hosted runners; you drive it with the MCP tools. The argument `$ARGUMENTS` selects the build profile: if it contains `release` (e.g. `/run-local release` or `release version`), use **release**; otherwise use the default **dev** profile.

Work through these phases in order, and never skip a phase.

## Phase 0 — Determine build mode
Set `MODE` from `$ARGUMENTS`:
- Contains `release` → `MODE=release`
- otherwise → `MODE=dev`

## Phase 1 — Runner pre-flight (STOP on failure)
1. Call `list_runners`. Print both machines and their status.
2. You need BOTH the Linux runner and the Windows runner **online**.
   - If the **Linux** runner is offline → **start it yourself** with `~/actions-runner/run.sh` (you are on Linux).
   - If the **Windows** runner is offline → **STOP and tell the user the exact command to run** on Windows: `C:\actions-runner\run.cmd`. Wait for them to confirm it's up, then re-run Phase 1.
   - Do not trigger anything until both runners are online.

## Phase 2 — Same-LAN gate (STOP and ASK)
1. Call `probe_network` to get each machine's public IP + LAN IP (it triggers the `network-probe` workflow and waits).
2. Compare the two `public_ip` values (fall back to `lan_ip` / subnet if a public IP is `unknown`).
   - If they **differ** (different networks) → proceed silently; this is the intended cross-network case.
   - If they **match** (same LAN/NAT) → **STOP and ask the user**, verbatim: **"You are on the same LAN! Was this intended? Yes/No"**.
     - Answer **Yes** → proceed.
     - Answer **No** → abort the whole command and tell the user to move one machine to a different network before re-running.

## Phase 3 — Trigger the pipeline
Call `run_pipeline` with:
- `crate` = `freenet_libp2p_bevy_example_1`
- `run_tests` = `true`
- `release_builds` = `true`
- `build_mode` = `MODE`

This runs: the full test gate on both Linux and Windows, the shared contract WASM build on Linux, the `MODE` binary build on both, and the cross-OS mainnet sync probe on both machines.

## Phase 4 — Poll to completion
Poll with `run_status` (and `list_runs` for an overview) until the run is finished. Do not report prematurely. Summarize each job's result (`resolve`, `test (linux)`, `test (windows)`, `build-contract`, `build (linux)`, `build (windows)`, `cross-os (linux)`, `cross-os (windows)`, `cross-os-verify`).

Note: with a single Linux runner the whole graph serialises — the Windows leg cannot start until Linux jobs free the runner, and the two `cross-os` legs need the Linux runner idle to run concurrently. Expect the Windows work to be queued behind Linux jobs.

## Phase 5 — Collect the bevy logs (the investigation artifact)
1. Determine the run id of the finished pipeline.
2. `download_artifacts` (pattern `cross-os-log-*`, dest `freenet_libp2p_bevy_example_1/logs/<run_id>/`) so you fetch both `cross-os-log-linux` and `cross-os-log-windows`.
3. Confirm exactly one `.log` per machine landed in that directory. These JSON-lines files are what the bevy app (embedded via the cross-OS test) reported — each row is `{"machine","own","observed","t"}`.

## Phase 6 — Final report
Print a concise report to the user:
- Job matrix with pass/fail (Phase 4 results), including the `cross-os-verify` PASS/FAIL verdict.
- The exact paths of the two `.log` files (single/double log for investigation).
- If `cross-os-verify` failed, quote from each log what each machine observed so the user can investigate the bug.
- Any runner issue you hit, and what the user must do manually (Windows: run `C:\actions-runner\run.cmd`; Linux: you start it yourself).

## Hard rules
- Never trigger the pipeline before Phase 1 confirms both runners online.
- Never skip the Phase 2 same-LAN question when public IPs match.
- Never use `--release` unless `$ARGUMENTS` asks for it — dev is the default and the point.
- Do not stage, commit, push, or trigger GitHub-hosted (tag) CI from this command.
