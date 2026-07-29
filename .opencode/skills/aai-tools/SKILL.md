---
name: aai-tools
description: |
  Use when working with the AAI tools MCP server (screenshots, process
  management). Covers the vision-check guard for screenshot calls and the
  full process lifecycle (spawn, read, write, kill, list).
---

# AAI Tools

## Screenshots

The `screenshot` MCP tool captures the primary monitor and returns a PNG
image plus a text summary (dimensions, file size).

**Vision guard (MANDATORY):** Before calling `screenshot`, check whether
the current model can handle images:

1. Find your model name from the system prompt in this session (it states
   "You are powered by the model named X"). Extract the bare model name
   after the last `/` (e.g. `qwen3.7-plus` from `opencode-go/qwen3.7-plus`).
2. Run:
   ```
   aai_mcp/scripts/aai-vision-check <model-name>
   ```
3. If the output is `vision: no`, **skip** the screenshot. Tell the user:
   "The current model (`<model-name>`) does not support image input.
   Screenshot skipped."
4. If `vision: yes`, proceed with the screenshot normally.

## Process Management

| Tool | Purpose |
|------|---------|
| `spawn_process` | Start a subprocess. Returns a numeric PID. |
| `read_output` | Drain buffered stdout/stderr collected since last call. |
| `write_stdin` | Send text to a running process (newline appended). |
| `kill_process` | Terminate a managed process and remove it. |
| `list_processes` | List all managed processes with IDs, commands, alive status. |

- PIDs are numeric and opaque — store them as given.
- `read_output` accepts an optional `timeout_ms` (default 200). Increase
  for slow-starting processes.
- `write_stdin` automatically appends a newline.
- Always `kill_process` when done with a subprocess to avoid leaks.
