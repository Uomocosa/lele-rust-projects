---
name: deskctrl
description: |
  Use when working with the deskctrl MCP server (screenshots, window
  listing, typing/clicking into windows, process management). Covers the
  vision-check guard for screenshot calls, capturing a single window
  instead of the whole screen, and the full process lifecycle (spawn,
  read, write, kill, list).
---

# deskctrl

MCP server registered under the key `deskctrl`, so its tools are
`mcp__deskctrl__*`. Source and registration docs: `deskctrl_mcp/README.md`.

## Screenshots

The `screenshot` tool returns a PNG image plus a text summary (dimensions,
file size). With no arguments it captures the whole screen.

**Session start:** take **one full-screen** screenshot before driving any
window. A modal dialog holding a keyboard grab (e.g. gnome-keyring's
"Choose password for new keyring") is hidden in a per-window capture but
visible in a full-screen shot — and it silently swallows every keystroke and
click until dismissed. `send_keys` now errors on an active grab rather than
typing into the grabber, but you still want to spot the dialog first.

**Vision guard (MANDATORY):** Before calling `screenshot`, check whether
the current model can handle images:

1. Find your model name from the system prompt in this session (it states
   "You are powered by the model named X"). Extract the bare model name
   after the last `/` (e.g. `qwen3.7-plus` from `opencode-go/qwen3.7-plus`).
2. Run:
   ```
   deskctrl_mcp/scripts/deskctrl-vision-check <model-name>
   ```
3. If the output is `vision: no`, **skip** the screenshot. Tell the user:
   "The current model (`<model-name>`) does not support image input.
   Screenshot skipped."
4. If `vision: yes`, proceed with the screenshot normally.

## Capturing a single window

Prefer this over a full-screen grab when you only care about one app — the
image is smaller and far easier to read.

1. Call `list_windows` to get the open windows: window id, owning pid,
   geometry, title.
2. Call `screenshot` with **one** selector:
   - `window_id` — e.g. `"0x03a00004"`. The reliable key; use it.
   - `pid` — convenience. Errors, listing the candidates, if the pid owns
     more than one window (browsers and Electron apps usually do).
   - `title` — case-insensitive substring. Same ambiguity rule.

If you get an ambiguity error, pick a `window_id` from the candidate list
in the message rather than guessing.

`list_windows` needs `wmctrl` and an X11 display. It is unrelated to
`list_processes`, which lists only subprocesses this server spawned.

## Clicking

`click_window` takes `window_id` plus `x`/`y` **relative to the window's
top-left** — the same coordinates you read off a `screenshot {window_id}`
image, so: screenshot, find the button, click those pixels.

It raises the window first (input is injected at the screen level, so an
overlapping window would otherwise swallow the click), which steals focus.
**Always screenshot the window afterwards** — a mis-aimed click lands on
empty space and fails silently.

## Typing / send_keys

`send_keys` types into the window focused after raising (same steal-focus rule as clicking).
Pass `window_id` plus `inputs`, a non-empty ordered list of deliberate keyboard actions. Each
element is one of:

- `tap` — `{"type":"tap","key":"a"}` — press and release one key.
- `hold` — `{"type":"hold","key":"d","duration_ms":1000}` — press a key and keep it down for the
  duration, then release. Auto-repeat turns a long hold into repeated characters, so **use this
  for "a run of d's" — never spell out `dddddd…`**.
- `chord` — `{"type":"chord","keys":["ctrl","shift","esc"]}` — press several keys together and
  release them all at once. Names are case-insensitive.
- `delay` — `{"type":"delay","duration_ms":300}` — wait without sending keys.
- `text` — `{"type":"text","text":"ls\n"}` — literal printable ASCII typed character by
  character (`\n` is Enter, `\t` is Tab). The keymap is read live from the X server, so shifted
  characters (`A`, `!`) follow the current layout.

A `key` is a modifier (`Ctrl`, `Shift`, `Alt`, `Super`, `Meta`), a named key (`Enter`, `Tab`,
`BackSpace`, `Escape`, `Delete`, `Insert`, `Home`, `End`, `PageUp`, `PageDown`, arrows,
`Space`, `F1`–`F12`), or a single printable ASCII character (letters are unshifted, so
`ctrl` + `a` means control-a).

Non-ASCII text errors — send Unicode through the clipboard instead. Screenshot the window
afterwards to confirm the text landed.

`send_keys` errors if another window (e.g. a modal dialog) holds an active keyboard grab —
keystrokes would go to the grabber, not the target. Dismiss the dialog and retry.

## Process Management

| Tool | Purpose |
|------|---------|
| `spawn_process` | Start a subprocess. Returns a numeric PID and the OS pid. |
| `read_output` | Return stdout/stderr collected since the last call. |
| `wait_for_output` | Block until a line containing a substring appears. |
| `write_stdin` | Send text to a running process (newline appended). |
| `kill_process` | Terminate a managed process and remove it. |
| `list_processes` | List all managed processes with IDs, commands, alive status. |

- PIDs are numeric and opaque — store them as given. They are *not* the
  same numbering as the OS pids shown by `list_windows`; `spawn_process`
  reports both, and the `os_pid` is what matches a window.
- To screenshot a GUI app you spawned, match its `os_pid` against
  `list_windows`. Two instances of one app have identical titles, so
  `title` is ambiguous and `window_id`/`pid` is the only way to tell them
  apart. Spawn the binary directly, never via `cargo run`, or the
  `os_pid` belongs to cargo rather than to the app.
- Prefer `wait_for_output` over polling `read_output` in a loop: it scans
  everything since spawn, so it cannot miss a line that an earlier
  `read_output` already returned. Its timeout is capped at 120 s per
  call — just call it again if the marker has not appeared yet.
- `read_output` accepts an optional `timeout_ms` (default 200). Increase
  for slow-starting processes.
- `write_stdin` automatically appends a newline.
- Always `kill_process` when done with a subprocess to avoid leaks.
