# test-orchestrator-mcp

MCP server that drives the cross-machine test pipeline for
`freenet-libp2p-bevy-example-1` from either PC. Run one copy on each machine
(Linux + Windows); both agent sessions get the same tools.

## Tools

| Tool | What it does |
|---|---|
| `list_runners` | self-hosted GitHub runner status (both machines, from either PC) |
| `run_pipeline` | trigger the self-hosted CI workflow on your machines (test gate + Linux/Windows release builds, one shared contract WASM) |
| `list_runs` / `run_status` | workflow run list / per-job status |
| `download_artifacts` | grab the built binaries (e.g. the Windows `.exe`) of a run |
| `next_tag` | preview the next `<crate>-<mode>-YYYY-MM-DD#N` tag (no push) |
| `trigger_tag_ci` | push that tag to start the GitHub-hosted crate-tag CI (`test`/`build`/`release`/`release-notests`) |
| `launch_game` | launch the game on THIS machine, detached, `RUST_LOG=warn,roster=info,p2p=info`, log to file |
| `game_status` | grep the game log for ring connections, roster entries, libp2p connections, errors |
| `stop_game` | terminate a launched game by pid |

## Setup (per machine)

1. `gh` CLI installed (`winget install GitHub.cli` on Windows) and `git`.
2. Build once: `cargo build --release` in this directory.
3. Copy `.env.example` to `.env` and fill in `GH_TOKEN` (see below).
4. Register the server with your agent (each machine uses its own absolute path).

   **opencode** — in `opencode.json` (project, or global `~/.config/opencode/opencode.json`):

```json
{
  "mcp": {
    "test-orchestrator": {
      "type": "local",
      "command": ["<absolute-path>/test_orchestrator_mcp/target/release/test-orchestrator-mcp"],
      "enabled": true
    }
  }
}
```

   **Claude Code** — in the project's `.mcp.json`:

```json
{
  "mcpServers": {
    "test-orchestrator": {
      "type": "stdio",
      "command": "<absolute-path>/test_orchestrator_mcp/target/release/test-orchestrator-mcp",
      "args": [],
      "env": {}
    }
  }
}
```

   Or equivalently:
   `claude mcp add --scope project test-orchestrator -- <absolute-path>/test_orchestrator_mcp/target/release/test-orchestrator-mcp`

   Leave `env` empty — `GH_TOKEN` belongs in `.env`, which the server loads from its
   own `CARGO_MANIFEST_DIR` regardless of the working directory it is started in.

5. Restart the agent; the tools appear automatically. (Claude Code will not pick up
   `.mcp.json` changes mid-session.)

## Secrets (`.env`, gitignored)

| Variable | Required | Meaning |
|---|---|---|
| `GH_TOKEN` | yes | classic PAT with `repo` + `workflow` scopes |
| `GH_REPO` | no | defaults to `Uomocosa/lele-rust-projects` |
| `FBX_GAME_EXE` | no | override the game binary path used by `launch_game` |

`GH_TOKEN` is only ever injected into `gh` subprocess environments; it is
never logged. When unset, the tools fall back to the machine's `gh` login.

## Windows runner + runtime test flow

1. Self-hosted pipeline: the agent on either PC calls `run_pipeline`; the
   Windows machine needs a registered self-hosted runner
   (`C:\actions-runner`, `config.cmd --labels windows`, `run.cmd`).
2. Runtime test (two different internets): each PC's agent calls
   `launch_game` locally (distinct `--p2p-port` per machine, explicit
   `--identity-dir` — the default path is `HOME`-only and unset on Windows),
   then `game_status` to watch for `ring_connections`, roster convergence and
   `ConnectionEstablished`.

**Do not add a firewall rule to make a test pass.** Hand-configuring inbound
UDP tests a special-cased machine rather than the software; real users will not
do it. Mainnet and libp2p are supposed to traverse NAT on their own, so a run
that fails without a manual rule is a finding to record, not something to work
around. See `freenet_libp2p_bevy_example_1/CROSS_NETWORK_TEST_2026-08-13.md`.

Note that a failed run leaks an in-process freenet node holding its UDP port
(the retry loop in `roster/connect_and_run.rs` never aborts the old one), and a
loopback port probe cannot see that binding. Run **one attempt per process
lifetime**: on a bootstrap failure or `EADDRINUSE` / WSA 10048, kill the process
and relaunch rather than letting it retry.
