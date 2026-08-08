# deskctrl-mcp

An MCP server that lets an agent drive this desktop: capture the screen or a single window,
list open windows, and spawn/steer/kill subprocesses.

Used by **both** opencode and Claude Code. Read the registration section before touching
anything — the two agents register it differently and neither picks it up automatically.

## Registration — you MUST do this for both agents

Three names are involved, and they are **not** the same name:

| Name | Value | Where it comes from |
|---|---|---|
| directory | `deskctrl_mcp` | this folder |
| crate / binary | `deskctrl-mcp` | `Cargo.toml` (`package.name`, `[[bin]].name`) |
| **MCP config key** | `deskctrl` | the config files below |

**The tool namespace comes from the config key, not the crate name.** Key `deskctrl` is why
the tools appear as `mcp__deskctrl__screenshot`, `mcp__deskctrl__list_windows`, … Rename the
key and every tool name an agent knows changes with it.

Both registrations point at the **release binary**, not `cargo run`:

```
deskctrl_mcp/target/release/deskctrl-mcp
```

So **`cargo build --release` is mandatory after any change.** A stale binary keeps serving the
old tool schemas — a newly added tool simply will not appear, with no error anywhere.

### opencode

Project-level `opencode.json` at the repo root, under `mcp`:

```json
"mcp": {
  "deskctrl": {
    "type": "local",
    "command": [
      "/home/uomocosa/Syncthing/[AAI] Agentic AI/rust_projects/projects/deskctrl_mcp/target/release/deskctrl-mcp"
    ],
    "enabled": true,
    "environment": {
      "AAI_ARTIFACTS_DIR": "/home/uomocosa/Syncthing/[AAI] Agentic AI/rust_projects/projects/deskctrl_mcp/artifacts/freenet_bevy"
    }
  }
}
```

Note opencode uses `command` as an **array** and `environment`.

### Claude Code

Repo-root `.mcp.json` (version-controlled, so a fresh clone is registered):

```json
{
  "mcpServers": {
    "deskctrl": {
      "type": "stdio",
      "command": "/home/uomocosa/Syncthing/[AAI] Agentic AI/rust_projects/projects/deskctrl_mcp/target/release/deskctrl-mcp",
      "args": [],
      "env": {
        "AAI_ARTIFACTS_DIR": "/home/uomocosa/Syncthing/[AAI] Agentic AI/rust_projects/projects/deskctrl_mcp/artifacts/claude_code"
      }
    }
  }
}
```

Claude Code uses `command` as a **string** plus `args`, and `env`. `~/.claude/settings.json`
already sets `"enableAllProjectMcpServers": true`, so this file is picked up with no further
action; otherwise Claude Code prompts to approve the project server on first start.

> **Required one-time manual step, outside the repo:** this server used to be registered
> globally in `~/.claude.json` under `mcpServers["aai-tools"]`, pointing at
> `aai_mcp/target/release/aai-mcp`. **That path no longer exists** — the crate was renamed.
> Delete that entry, or every Claude Code session starts by failing to launch a dead binary.

Restart the agent after any registration or binary change — MCP servers are launched at
session start.

## Tools

| Tool | What it does |
|---|---|
| `list_windows` | Open desktop windows: window id, owning pid, geometry, title. Backed by `wmctrl -l -p -G`. |
| `screenshot` | Whole screen with no arguments; one window with `window_id`, `pid`, or `title`. |
| `click_window` | Click inside a window at window-relative coordinates. |
| `list_processes` | **Not** windows — only the subprocesses this server spawned. |
| `spawn_process` / `read_output` / `write_stdin` / `kill_process` | Managed subprocess control. |
| `wait_for_output` | Block until a spawned process prints a line containing a substring. |
| `send_to_telegram` | Send text and/or a PNG to the configured chat. |

### Driving a GUI app

`spawn_process` returns both its own `id` (used by the other process tools) and the **`os_pid`**,
which is the number `list_windows` reports for the window. That is the only reliable way to tell
two instances of the same app apart — their window titles are identical, so `screenshot {title}`
correctly refuses as ambiguous.

Spawn the binary **directly, never via `cargo run`** — otherwise `os_pid` is cargo's pid and the
window belongs to a grandchild, so the correlation silently breaks.

`click_window` takes coordinates **relative to the window's top-left**, i.e. the same coordinates
you read off a `screenshot {window_id}` image. It raises the window first (XTEST injects input at
the root, so an overlapping window would otherwise swallow the click) — this steals focus. Always
`screenshot` the window afterwards to confirm the click landed; a mistranslated click hits empty
space and fails silently.

### Waiting on a slow process

`read_output` returns only what is new since the last call. `wait_for_output` instead scans the
**entire transcript**, so it still finds a line an earlier `read_output` already returned — use it
for readiness markers rather than polling `read_output` in a loop and risking losing the line.

Its timeout is capped at **120 s per call**; if the marker has not appeared, call again. This
keeps a single call under the MCP client's own per-tool timeout. Raise `MCP_TOOL_TIMEOUT` if you
would rather make one long call. Per-process output is retained up to 4 MB, trimmed from the
oldest whole lines.

### Targeting a window

`screenshot` takes three optional selectors; the first non-null wins, in order:

1. `window_id` — `"0x03a00004"` from `list_windows`. The stable key.
2. `pid` — convenience; **errors with the candidate list** if the pid owns more than one window
   (browsers and Electron apps usually do).
3. `title` — case-insensitive substring; same ambiguity rule.

With none set the behaviour is exactly as before: full screen, preceded by the screen-wake
sequence and a 5s settle. The targeted path skips that wake/settle — a window addressed by id
is known to exist already.

Capture chain for a window, in order: `import -window <id>` (under a compositor this returns
the window's own pixels even when occluded) → `xwd -id <id>` → as a last resort raise it with
`wmctrl -i -a <id>` and crop the root grab to its geometry. The raise is last because it steals
focus.

## Requirements

X11 only (`DISPLAY` must be set) — there is no Wayland path. External binaries:

- `wmctrl` — required for `list_windows` and all window targeting
- `import` + `convert` (ImageMagick), `xwd`, `gnome-screenshot` — capture, tried in that order
- the X server's **XTEST** extension, for `click_window` — spoken directly via the pure-Rust
  `x11rb` crate, so there is nothing to install
- `xdotool` is **not** used and is not installed on this machine; do not add a dependency on it

### Known limitations

- `list_windows` hides the window titled exactly `Desktop` — that is the file manager drawing
  the wallpaper, and it is never what you want to capture. A real window with that exact title
  is hidden too; target it by `pid` instead.
- The `pid` in `list_windows` is the OS pid from `_NET_WM_PID`. It is `0` for windows that do
  not publish one, and it is unrelated to the ids handed out by `spawn_process`.

## Configuration

`.env` next to `Cargo.toml` (see `.env.example`), loaded via `CARGO_MANIFEST_DIR` so the
working directory does not matter. That bakes this checkout's absolute path into the binary —
fine here, but if the tree moves (it lives under Syncthing), rebuild. A `.env` in the working
directory is used as a fallback:

- `AAI_ARTIFACTS_DIR` — if set, every screenshot is also written to `<dir>/<unix_secs>.png`.
  Each agent points at its own subdirectory of `artifacts/` so their captures do not interleave.
- `TELEGRAM_BOT_TOKEN`, `TELEGRAM_CHAT_ID` — if both set, screenshots are also pushed to
  Telegram, fire-and-forget.

## Adding a tool

1. Implement it as one file in `src/ServerMethod/`, and export it from
   `src/ServerMethod/mod.rs`. One public function per file is the convention throughout.
2. If it takes arguments, add a params struct in its own `src/PascalCase.rs` file
   (`#[derive(Deserialize, schemars::JsonSchema)]`, doc comments become the schema
   descriptions) and wire it into `src/main.rs`.
3. Add one `#[tool(description = "…")]` line in `src/Server.rs`.
4. Mention it in the instructions string in `src/ServerMethod/get_info.rs`.
5. `cargo fmt && cargo clippy --all-targets && cargo test`, then **`cargo build --release`**.
6. Restart both agents and confirm the tool appears.

Tests that need a live X display are `#[ignore]`d; run them with
`cargo test -- --ignored`. Everything else (`wmctrl` line parsing, id validation, PNG header
parsing) is covered headlessly against fixture strings.
