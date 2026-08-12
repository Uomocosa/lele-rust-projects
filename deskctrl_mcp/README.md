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
| `send_keys` | Send a deliberate sequence of keyboard inputs (tap / hold / chord / delay / text) into a window (XTEST). |
| `list_processes` | **Not** windows — only the subprocesses this server spawned. |
| `spawn_process` / `read_output` / `write_stdin` / `kill_process` | Managed subprocess control. |
| `wait_for_output` | Block until a spawned process prints a line containing a substring. |
| `send_to_telegram` | Send text and/or a PNG to the configured chat. |
| `record_video` | Start/stop an ffmpeg screen recording and send the MP4 to Telegram. |

## Telegram: step-by-step notifications

`send_action_summary` is gone. Instead of one big report, the server pushes **step-by-step
messages** to Telegram as you work, plus a session-start banner and a session-end video.

- On session start the server sends `📋 Starting Session - YYYY_MM_DD [hh:mm:ss]` and begins an
  ffmpeg recording of the screen (capped at `RECORDING_MAX_SECS`, default 10 minutes, so the file
  stays well under Telegram's 50 MB upload limit).
- Each **visible-action** tool — `screenshot`, `click_window`, `send_keys`, `spawn_process`,
  `write_stdin`, `kill_process`, `record_video` — accepts a `send_to_telegram` flag that
  **defaults to `true`**. When true (and Telegram is configured) the tool pushes its own short
  message:
  - `screenshot` sends the photo with a caption — pass `caption` (e.g.
    `"freenet clicker state now at 8"`), or it falls back to an auto summary (target + size).
  - `click_window` sends a text message — pass `note` (e.g.
    `"clicking 'Increment button', expected in the image: freenet clicker state now at 8"`),
    or it auto-describes the click.
  - `send_keys` sends a text message — pass `note` (e.g.
    `"typing 'ls' in xterm, expected in the image: the prompt shows the typed command"`),
    or it auto-describes the input plan.
  - `spawn` / `write_stdin` / `kill` send an auto template from their arguments.
- The `send_to_telegram` flag is how the agent keeps the feed from flooding: leave it `true` only
  for steps with visible impact, set it `false` for routine/read-only calls (`list_windows`,
  `read_output`, `wait_for_output`, `list_processes` have no flag and never notify).
- On session end (the stdio transport closes) the recording is stopped and the MP4 is uploaded to
  Telegram with a caption; the upload is awaited (with a 60s timeout) before the server exits, and
  a Telegram text is sent instead if the video is too large or the upload fails.

Requires `ffmpeg` and `xdpyinfo` for recording. If `ffmpeg` is missing, `record_video` returns an
error asking you to install it; the session-end auto-send is skipped with a warning.

### Driving a GUI app

**Start each session with one full-screen `screenshot`.** A modal dialog that holds a keyboard
grab (e.g. gnome-keyring's "Choose password for new keyring") hides at a glance behind a
per-window capture, but shows up in a full-screen shot — and it will silently swallow every
keystroke/click until dismissed. Only the full-screen capture reveals it, so take one at session
start and watch for anything unexpected before driving any window.

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

`send_keys` types into the window that is focused after raising, with the same steal-focus caveat.
It takes `window_id` plus `inputs`, a non-empty ordered list of deliberate keyboard actions. Each
element is one of:

- `tap` — `{ "type": "tap", "key": "a" }`. Press and release one key.
- `hold` — `{ "type": "hold", "key": "d", "duration_ms": 1000 }`. Press a key and keep it down
  for the duration, then release. X keyboard auto-repeat turns a long hold into repeated
  characters, so this is how you type "a run of d's" — never spell out `dddddd…`.
- `chord` — `{ "type": "chord", "keys": ["ctrl", "shift", "esc"] }`. Press several keys together
  and release them all at once. Names are case-insensitive.
- `delay` — `{ "type": "delay", "duration_ms": 300 }`. Wait without sending any keys.
- `text` — `{ "type": "text", "text": "ls\n" }`. Literal printable ASCII typed character by
  character (`\n` is Enter, `\t` is Tab).

A `key` is a modifier (`Ctrl`/`Control`, `Shift`, `Alt`, `Super`/`Win`, `Meta`), a named key
(`Enter`/`Return`, `Tab`, `BackSpace`, `Escape`/`Esc`, `Delete`/`Del`, `Insert`/`Ins`, `Home`,
`End`, `PageUp`, `PageDown`, the arrows, `Space`, `F1`–`F12`), or a single printable ASCII
character.

Sending is **deliberate and bounded**: the whole sequence is validated before any key is pressed,
the plan is capped (a few thousand units / at most 120 s of holds+delays), and any keys held by a
`chord`/`hold` are always released — even if a later step errors — so a stuck modifier can never
hang the desktop. Each press/release is flushed to the X server immediately, so `hold` durations
and `delay`s have real timing (auto-repeat fires during a long hold). `send_keys` first probes for
an active keyboard grab by another window (e.g. a modal dialog): if one is held, it **errors
instead of typing into the grabber** — `raise_window`'s focus check alone cannot detect this.
Non-ASCII text errors — send Unicode through the clipboard instead. Screenshot the window
afterwards to confirm the text landed.

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
- the X server's **XTEST** extension, for `click_window` and `send_keys` — spoken directly via
  the pure-Rust `x11rb` crate, so there is nothing to install
- `xdotool` is **not** used and is not installed on this machine; do not add a dependency on it
- `ffmpeg` + `xdpyinfo` — required only for `record_video` (screen capture via `x11grab`)

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
  Recordings are written here as `<unix_secs>.mp4` too (else `/tmp`).
- `RECORDING_MAX_SECS` — max recording length in seconds before ffmpeg stops itself (default
  `600`, i.e. 10 minutes). Keeps session videos small enough to send via Telegram.
- `TELEGRAM_BOT_TOKEN`, `TELEGRAM_CHAT_ID` — if both set, notifications, session start/end
  messages, screenshots and the session video are pushed to Telegram.

## Adding a tool

This crate is `lele_lint`-clean: snake_case filenames everywhere, one public item per file,
and a `test_usage` test (or `// no test_usage necessary`) in every file. Run
`cargo run --manifest-path ../lele_lint/Cargo.toml` after changes.

1. Implement it as one file in `src/server_method/`, and export it from
   `src/server_method/mod.rs`. One public function per file is the convention throughout.
2. If it takes arguments, add a params struct in its own `src/<snake_case>_params.rs` file
   (`#[derive(Deserialize, schemars::JsonSchema)]`, doc comments become the schema
   descriptions) and wire it into `src/main.rs`.
3. Add one `#[tool(description = "…")]` line in `src/server.rs` (a thin 1-liner delegate; any
   multi-statement logic goes in a `src/server_<method>.rs` method file or the action's
   `server_method` file).
4. Mention it in the instructions string in `src/server_method/get_info.rs`.
5. `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`, run `lele_lint`,
   then **`cargo build --release`**.
6. Restart both agents and confirm the tool appears.

Tests that need a live X display are `#[ignore]`d; run them with
`cargo test -- --ignored`. Everything else (`wmctrl` line parsing, id validation, PNG header
parsing) is covered headlessly against fixture strings.
