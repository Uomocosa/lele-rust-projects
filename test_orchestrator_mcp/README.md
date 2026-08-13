# test-orchestrator-mcp

MCP server that drives the cross-machine test pipeline for
`freenet-libp2p-bevy-example-1` from either PC. Run one copy on each machine
(Linux + Windows); both opencode sessions get the same tools.

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
4. Register the server in opencode's **global** config
   `~/.config/opencode/opencode.json` (each machine uses its own absolute path):

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

5. Restart opencode; the tools appear automatically.

## Secrets (`.env`, gitignored)

| Variable | Required | Meaning |
|---|---|---|
| `GH_TOKEN` | yes | classic PAT with `repo` + `workflow` scopes |
| `GH_REPO` | no | defaults to `Uomocosa/lele-rust-projects` |
| `FBX_GAME_EXE` | no | override the game binary path used by `launch_game` |

`GH_TOKEN` is only ever injected into `gh` subprocess environments; it is
never logged. When unset, the tools fall back to the machine's `gh` login.

## Windows runner + runtime test flow

1. Self-hosted pipeline: opencode on either PC calls `run_pipeline`; the
   Windows machine needs a registered self-hosted runner
   (`C:\actions-runner`, `config.cmd --labels windows`, `run.cmd`).
2. Runtime test (two different internets): each PC's opencode calls
   `launch_game` locally (distinct `--p2p-port` per machine), then
   `game_status` to watch for `ring_connections`, roster convergence and
   `ConnectionEstablished`. Windows needs an inbound UDP firewall rule for
   its fixed `--p2p-port` (hotspot networks default to blocked inbound).
