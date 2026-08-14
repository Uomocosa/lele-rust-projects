# Two-machine test — 2026-08-14

Follow-up to `CROSS_NETWORK_TEST_2026-08-13.md`. **The deliverable this time is
the MCP + self-hosted-runner pipeline**, not the game's network behaviour; the
network numbers are recorded as data, not as a pass/fail gate.

Constraints held throughout: **no firewall rules**, **no code changes**, public
mainnet only (no `--freenet-local`, no `--freenet-gateway`).

## Verdict up front

**The Linux half of the pipeline works end to end. The Windows half still does
not, for a second, separate reason.**

The Windows leg had been failing 21 s into every run on a runner-environment
defect (P1, WSL bash shadowing Git Bash) — meaning the Windows half of this
pipeline had *never actually run*. Fixing that got it compiling, at which point
it hit P7: still building after **52 minutes** against Linux's 8, and it was
cancelled. **No Windows binary was produced and no Windows game instance was
started**, so the cross-machine game data this run was meant to collect does
not exist.

Seven problems were found, all in the pipeline and tooling, none in the game:
P1/P7 (Windows CI), P2/P3 (runner workspace), P4/P6 (MCP `launch_game`),
P5 (flaky test gate).

**This run does not test cross-network traversal.** Both machines are on the
same router and the same WiFi SSID (`PosteMobile-83429328`, gateway
`192.168.1.1`). The 2026-08-13 report flagged this premise as unconfirmed; this
time it is confirmed to be a **same-LAN run**, so no NAT/traversal conclusion
can be drawn from it either way.

---

## Environment

| | Linux desktop (this box) | Windows box |
|---|---|---|
| runner | `uomocosa-desktop-linux` | `simone-pc-windows` |
| runner version | 2.336.0 | 2.336.0 |
| LAN | `192.168.1.9` (wlp8s0) | same subnet, same AP |
| WiFi | `PosteMobile-83429328` | same |
| gateway | `192.168.1.1` | same |
| public IP | `94.33.34.31` | same (same NAT) |
| Tailscale | `100.113.107.37` | — |

Both runners were **offline** at the start — a reboot killed both, since neither
is installed as a service (Linux is a foreground `~/actions-runner/run.sh`, no
systemd unit). Restarting them was the first step of the run.

---

## Pipeline result

Two runs on commit `4a4921e`, both via the `test-orchestrator` MCP
(`run_pipeline` → `run_status`).

| job | run #31780515957 (pre-fix) | run #31780978263 (post-fix) |
|---|---|---|
| `resolve` | ✅ success | ✅ success |
| `build-contract` | ✅ success | ✅ success |
| `test` | ✅ success | ❌ **failure (flaky)** |
| `build (linux)` | ✅ success | ✅ success (~8 min) |
| `build (windows)` | ❌ **failure (WSL bash, 21 s)** | ⏹ **cancelled after 52 min, still compiling** |

Runs: <https://github.com/Uomocosa/lele-rust-projects/actions/runs/31780515957>,
<https://github.com/Uomocosa/lele-rust-projects/actions/runs/31780978263>

### Shared contract artifact — verified on Linux

The `contract-wasm` fan-out works as designed on the Linux leg:

```
Preparing to download the following artifacts:
- contract-wasm (ID: 9211513534, Size: 41669,
  Expected Digest: sha256:26949f2199d8a3238bcbb7a5eaf8ea4f99656847dc049b6d54724608ce1c3dc9)
Starting download of artifact to: .../contract-wasm-download
SHA256 digest of downloaded artifact is 26949f2199...
Artifact download completed successfully.
Run src=$(find "$GITHUB_WORKSPACE/contract-wasm-download" -name '*.wasm' | head -n1)
```

One `build-contract` job produces the WASM; each `build` leg downloads it,
digest-verified, and copies it to the embed location. `CARGO_TARGET_DIR:
target/ci` is honoured — the upload step resolved
`freenet_libp2p_bevy_example_1/target/ci/release/freenet-libp2p-bevy-example-1`
with `if-no-files-found: error` not tripping.

---

## Pipeline findings

### P1 — Windows runner resolves `bash` to WSL, breaking every Windows job (critical, fixed)

`dtolnay/rust-toolchain@stable` runs its steps with
`shell: C:\WINDOWS\system32\bash.EXE`. That is **WSL bash**, which cannot read a
Windows path — the backslashes are silently eaten:

```
shell: C:\WINDOWS\system32\bash.EXE --noprofile --norc -e -o pipefail {0}
/bin/bash: C:actions-runner_work_temp6ef91329-6eb2-4e2d-b474-772fe02e9cd2.sh: No such file or directory
##[error]Process completed with exit code 1
```

The job died at toolchain-parse, 21 s in — before checkout of the contract
artifact, before any cargo work. `CARGO_TARGET_DIR: target/ci` was correctly set
in the job env but nothing ever used it.

`where bash` on the Windows box:

```
C:\Windows\System32\bash.exe                                  ← WSL, wins
C:\Users\acer\AppData\Local\Microsoft\WindowsApps\bash.exe
C:\msys64\usr\bin\bash.exe
```

Git for Windows *was* installed (`C:\Program Files\Git\bin\bash.exe` exists) but
its `bin` directory was not on PATH. **Fix applied:** start the runner from a
shell with `set PATH=C:\Program Files\Git\bin;%PATH%`, so Git Bash resolves
first. After this the Windows leg proceeded past the toolchain step normally.

This is environmental, not a repo change — but it means **the Windows half of
this pipeline had never actually run** until today. Worth making durable
(machine-level PATH, or install the runner as a service with a fixed PATH)
rather than depending on whoever starts `run.cmd` remembering to set it.

### P2 — a single Linux runner serialises the entire job graph

`build (windows)` depends on `resolve`, and `resolve` is Linux-only. With one
Linux runner, a second pipeline cannot start its Windows leg until the first
run's Linux build finishes. Run #2 sat fully queued for ~13 minutes behind run
#1's release build, despite the Windows runner being idle the whole time. If
Windows-leg turnaround matters, `build (windows)` should not transit a
Linux-only job.

### P3 — a second run destroys the first run's binaries (data-loss trap)

`actions/checkout` runs with `clean: true` (`git clean -ffdx`), which deletes
ignored files — including `target/`. Triggering run #2 wiped run #1's
successfully built `target/ci/release/freenet-libp2p-bevy-example-1` off disk.

This directly invalidates the assumption that "this box is the Linux runner, so
the fresh binary is already there": it is there **only until the next run's
checkout**. Anything wanting that binary must either grab it before the next
trigger or pull the uploaded artifact.

### P4 — `launch_game` drops the caller's `exe` parameter

Passing an explicit `exe` had no effect; the tool still resolved its default
path and failed on it:

```
launch_game(exe="/home/uomocosa/actions-runner/_work/.../target/ci/release/freenet-libp2p-bevy-example-1", …)
→ game executable not found at
  /home/…/projects/test_orchestrator_mcp/../freenet_libp2p_bevy_example_1/target/ci/release/freenet-libp2p-bevy-example-1
```

The reported path is the default, not the one supplied.

Root cause is exact and one line. `server.rs:53`:

```rust
async fn launch_game(&self, Parameters(params): Parameters<LaunchGameParams>) -> ... {
    server_method::launch_game(self.game_exe.as_deref(), params).await ...
}
```

The `exe_override` argument is filled from the **server's own `game_exe`
config**, so `params.exe` — the field the JSON schema advertises to callers — is
deserialized and then silently dropped. `default_exe()` in
`server_method/launch_game.rs:42-62` honours its argument correctly; the value
just never reaches it. The tool's own test encodes the bug rather than catching
it (`launch_game(None, params)` with `params.exe = Some("/nonexistent/fbx")`,
asserting the *default* path errors).

Suggested fix: `params.exe.as_deref().or(self.game_exe.as_deref())`.

This matters because CI builds into the **runner workspace**
(`~/actions-runner/_work/lele-rust-projects/lele-rust-projects/…`) while the MCP
resolves relative to the **Syncthing checkout**, where
`freenet_libp2p_bevy_example_1/target/ci/release/` does not exist at all. So
`exe` is the only way to reach a CI-built binary, and it does not work. Worked
around here with a symlink from the MCP's expected path to the runner workspace
(binary md5 `6fab003f49f7e69e9bc4db1aacea6310`; the symlink dangles as soon as
a run #3 checkout fires — see P3).

### P5 — the `test` gate is flaky

Identical commit, opposite results: `test` passed in run #1 and failed in run #2.

```
test two_nodes_see_each_other_in_the_roster ... FAILED
thread 'two_nodes_see_each_other_in_the_roster' panicked at tests/two_node_roster.rs:60:14:
gateway should observe both roster entries
test result: FAILED. 0 passed; 1 failed; 0 ignored
```

`testing/tests/two_node_roster.rs:57-60` waits up to 30 s for the gateway to
receive the peer's roster `UpdateNotification`; that notification did not arrive
in time. All 56 unit tests passed in both runs — only this integration test is
non-deterministic. Consequence for the pipeline: a red `test` does not reliably
indicate a regression, which undermines the gate's value.

### P7 — the Windows release build does not finish in a usable time

After P1 was fixed, `build (windows)` got past the toolchain step and compiled
normally — but was **still running after 52 minutes** (07:53:24Z → 08:45Z) and
was cancelled. For comparison, `build (linux)` on the same commit and the same
shared contract artifact took **~8 minutes** (07:40 → 07:48).

The runner console showed only `Running job: build (windows)` with no error, and
GitHub does not serve logs for a job still in progress, so there is no
step-level breakdown of where the time went. What can be said:

- it is a genuinely cold build (P3's `git clean -ffdx` wipes `target/` between
  runs, so **every** run on this pipeline is a cold build — there is no
  incremental reuse to be had);
- a >6× gap versus Linux on the same workload points at the Windows box's disk
  or antivirus scanning `target/ci`, not at the workflow definition.

**Consequence for this test: the Windows game instance was never started.** The
binary did not exist, so steps 4–6 of the plan could not be executed on Windows.

This is the single biggest obstacle to the two-machine workflow being practical:
one Windows-side change costs the better part of an hour before anything can be
tested. Worth investigating (exclude `target/ci` from Defender, or persist the
Windows cargo cache across runs by not cleaning `target/`) before the next
cross-machine attempt.

### P6 — `launch_game`'s log filter hides the metric `game_status` advertises

`launch_game` hardcodes the child's environment
(`server_method/launch_game.rs:29`):

```rust
.env("RUST_LOG", "warn,roster=info,p2p=info")
```

Every `freenet::*` target is therefore capped at WARN. `game_status`'s
documented job is to report "ring connections … libp2p connections", but under
this filter freenet never emits a healthy ring-connection count — all 28
`freenet::ring` lines in this run are WARN-level `prune_connection` notices, and
`ring_connections=` appears **zero** times.

Two consequences, in opposite directions:

- **Failure is still detectable.** `Zero ring connections detected` and
  `RING_TRANSPORT_DESYNC` are warnings, so the 2026-08-13 Windows symptom would
  still show up. Their absence here is meaningful.
- **Health is not measurable.** A positive ring count cannot be observed at all,
  so "26 → N ring connections" cannot be compared across the two reports.

This also **retracts** any reading of the 2800 → 199 line drop as an improvement
to baseline finding 8. No code changed between the runs. The difference is that
`launch_game` sets `RUST_LOG` explicitly, whereas the 2026-08-13 run was started
by hand and hit the override finding 8 describes. Same cause, opposite sign: the
noise went away and took the ring metric with it.

---

## Game run — network data

### Linux instance

Launched via `launch_game` (pid 233319) from the run-#2 CI binary:

```
identity_dir = /tmp/fbx_linux_mainnet
p2p_port     = 31337
log          = /home/uomocosa/Downloads/fbx_linux.log
```

| metric | 2026-08-13 | 2026-08-14 |
|---|---|---|
| embedded node ready | — | **1.1 s** |
| roster entries | 3 | **6** (5 existing + own) |
| ring connections | 26 | **not measurable — see P6** |
| `RING_TRANSPORT_DESYNC` | not on Linux | **none** |
| keepalive `CONNECTION TIMEOUT` | — | **6** |
| log volume | 2800 lines / 1.2 MB in ~8 min | 199 lines in ~15 min (**filter artefact, see P6**) |

The one connection count visible in this run is `active_connections=27`, and it
appears only incidentally, inside an unrelated `p2p_protoc` WARN. It is **not**
the same metric as the baseline's "26 ring connections" and the two should not
be read as a like-for-like improvement.

Startup was clean and fast:

```
08:11:03 roster: starting in-process network-mode node ws_port=35017 public_port=31337
                 skip_load_from_network=false is_gateway=false gateway=None
08:11:04 roster: embedded node ready; public_port=31337
08:11:04 roster: roster GetResponse existing_len=5 already_present=false
08:11:04 roster: merging own entry, sending roster Update merged_len=6
08:11:04 roster: received roster GetResponse (refresh) entries=6
```

The roster then held at 6 entries for the whole run, with **both** pull
refreshes (every 5 s) and push `UpdateNotification`s arriving — the commutative
merge and the notification path both work on mainnet.

libp2p identity: `12D3KooWRw1xEJ23DRyDhgAJahnZBxgn8CvF1i47eqr11cwtbaNm`
(fresh, written to `/tmp/fbx_linux_mainnet/identity.bin`).

Errors were all third-party mainnet churn, e.g.

```
ERROR freenet::transport::connection_handler: Failed NAT traversal
      error=... max connection attempts reached peer_addr=102.0.28.134:45692
```

Notably, the same two gateways as the 2026-08-13 Windows failure went silent on
this healthy Linux node too, without harming it:

```
WARN freenet_core::transport::keepalive_timeout: CONNECTION TIMEOUT -
     no packets received for 121.79s remote=5.9.111.215:31337
WARN freenet_core::transport::keepalive_timeout: CONNECTION TIMEOUT -
     no packets received for 123.19s remote=100.27.151.80:31337
```

So gateway keepalive loss is **not by itself** diagnostic of the Windows
failure — it happens on a node with 27 active connections and a fully synced
roster.

Final counts over the full 34-minute run (08:11:02 → 08:45:05, 501 lines /
132 KB, 186 of them `roster`):

| signal | count |
|---|---|
| `RING_TRANSPORT_DESYNC` | **0** |
| `Zero ring connections` | **0** |
| `retrying embedded node startup` | **0** |
| `Failed to bind UDP socket` / EADDRINUSE | **0** |
| keepalive `CONNECTION TIMEOUT` | 7 |
| `Failed NAT traversal` (third-party peers) | 30 |
| final roster entries | **6** |

The port-leak retry deadlock (baseline finding 1) **did not trigger**: the node
started first time, so `connect_and_run.rs`'s retry loop was never entered and
no freenet node or temp dir was leaked. This does not clear the bug — it means
the conditions that expose it did not occur on Linux.

### Windows instance

**Not started.** The Windows release build never produced a binary (P7), so
there was nothing to launch. No Windows-side data exists for this run: no ring
connection count, no roster count, no `RING_TRANSPORT_DESYNC` observation.

The 2026-08-13 question — *does Windows sit at 0 ring connections, and does that
implicate the inert `autonat`/`dcutr`/`relay::client` stack in
`src/p2p/behaviour.rs`?* — therefore **remains open and untested**.

---

## Baseline findings re-checked

| 2026-08-13 finding | status today |
|---|---|
| 8 — freenet INFO drowns the signal | **not retested**: the quieter log is `launch_game`'s explicit `RUST_LOG`, not a fix (P6) |
| 9 — Bevy LogPlugin fights the subscriber | **still present**: `Could not set global logger and tracing subscriber` logged once at startup |
| 1a/1b — port leak / bad UDP probe | **not exercised on Linux** (no retry was needed) |
| 2 — inert libp2p NAT stack | unchanged (no code changes permitted) |

---

## What worked

- **The MCP drove the whole pipeline.** `list_runners`, `run_pipeline`,
  `run_status`, `list_runs`, `launch_game`, `game_status` and `stop_game` all
  did their jobs; the two defects found (P4, P6) are in `launch_game`'s
  parameter handling and log filter, not in the tools' basic operation.
- **The shared-contract design.** One `build-contract` job, one WASM,
  digest-verified fan-out to each build leg.
- **The Linux half end to end**: CI build → launch → mainnet join → 6-entry
  roster → clean stop, with no retries and no leaks.
- **The roster contract on mainnet**, again: `existing_len=5` → `merged_len=6`,
  held at 6 for 34 minutes across both pull refresh and push notification.

## Cleanup — done

- Linux game pid 233319 stopped via `stop_game`. ✅
- Run #31780978263 cancelled while `build (windows)` was mid-compile. ✅
- **No leaked freenet nodes** (`pgrep freenet` → empty) and **no leaked temp
  dirs** (`/tmp/.tmp*` → empty). The plan anticipated both; neither occurred,
  because no bootstrap attempt failed. ✅
- Persisted identity left in place at `/tmp/fbx_linux_mainnet/identity.bin`.

Still on disk, deliberately:

- `~/Downloads/fbx_linux.log` — 501 lines, the evidence base for this report.
- Symlink `freenet_libp2p_bevy_example_1/target/ci/release/freenet-libp2p-bevy-example-1`
  → runner workspace (the P4 workaround). It **dangles as soon as the next CI
  run's checkout fires** (P3); delete it if it confuses a local build.
- `contract-wasm-download/` may remain in the runner workspace.

## Next steps, in order

1. **P7** — find out why the Windows build takes >6× the Linux build. Until
   this is fixed the two-machine loop is impractical, and every other Windows
   question stays blocked behind it.
2. **P1 durability** — make the Git-Bash-before-WSL PATH permanent on the
   Windows box (machine PATH or a runner service), so it survives the next
   reboot rather than depending on how `run.cmd` was started.
3. **P4** — one-line fix so `launch_game`'s `exe` parameter works.
4. **P6** — let `game_status` see a ring count, or stop advertising one.
5. **P5** — stabilise or quarantine `two_nodes_see_each_other_in_the_roster`.
6. Only then re-attempt the cross-network test — **on genuinely different
   networks**, which this run was not.
