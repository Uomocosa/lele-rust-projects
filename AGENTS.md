# AGENTS RULES

Welcome. You are an expert Rust software engineer. **This workspace is Rust-only. All generated code, suggestions, and architecture decisions assume Rust. Skills are scoped for Rust projects.**

Read [OBJECTIVE.md](./OBJECTIVE.md) for the project's goals, constraints, and current phase. Look at `src/` to see the existing module hierarchy. Do not invent root modules.

---

## Project Commands

| Key | Command |
|-----|---------|
| `RUN_ALL_TESTS` | `cargo build --workspace --all-targets && cargo clippy --workspace -- -D warnings && cargo fmt -- --check && cargo nextest run --all-targets && cargo run --manifest-path ../lele_lint/Cargo.toml` |
| `RUN_BUILD_CLIPPY` | `cargo build --workspace --all-targets && cargo clippy --workspace -- -D warnings` |
| `RUN_LELE_LINT` | `cargo run --manifest-path ../lele_lint/Cargo.toml` |

> `bacon clippy -- -- -D warnings` is **USER-ONLY** — agents NEVER run `bacon` (TUI, user tool). Agents use `cargo clippy -- -D warnings` via `devenv tasks run`.

## Devenv Tasks — MANDATORY (CRITICAL)

**If a crate has `devenv.nix` with `tasks`, you MUST use `devenv tasks run <task> 2>&1` — NEVER run the underlying `cargo ...` command by hand. Tasks exist to be tested and kept working — bypassing them defeats their purpose.**

1. **Read `<crate>/devenv.nix` first** on every task to discover the canonical `tasks."<ns>:<name>".exec`. Those `exec` strings are the single source of truth.
2. **Invoke via tasks, not raw cargo:** e.g. `devenv tasks run lele:clippy 2>&1`, `devenv tasks run lele:fmt 2>&1`, `devenv tasks run lele:nextest 2>&1`, `devenv tasks run lele:lint 2>&1`. Raw `cargo build/clippy/fmt/nextest` is ONLY allowed as fallback when `devenv.nix` is absent or for explicit shell-isolation checks (`devenv shell -- cargo ...`).
3. **Always append `2>&1` on the caller** (`devenv tasks run <task> 2>&1`). `cargo` writes diagnostics to `stderr`; without merging the LLM/tool sees empty output. Do NOT inline `2>&1` into `tasks.*.exec`.
4. **NEVER pipe to `| tail`, `| head`, `| grep`, or any pipe** on `devenv tasks run`. Fresh `cargo` with `showOutput = true` streams correctly; pipes swallow output and `tail` hangs 120s with `(no output)`. Use `showOutput = true` in task definitions, bare `devenv tasks run <task> 2>&1` on caller.

## Standard Build & Verification Routine

Verify changes with:
```bash
cargo build --all-targets
cargo clippy -- -D warnings
cargo fmt -- --check
cargo nextest run --all-targets
cargo run --manifest-path ../lele_lint/Cargo.toml
```
Via devenv (per-crate `devenv.nix`):
```bash
devenv tasks run lele:build 2>&1
devenv tasks run lele:clippy 2>&1
devenv tasks run lele:fmt 2>&1
devenv tasks run lele:nextest 2>&1
devenv tasks run lele:lint 2>&1
# fallbacks when devenv is absent or for shell isolation checks:
devenv shell -- cargo build --all-targets 2>&1
devenv shell -- cargo clippy -- -D warnings 2>&1
devenv shell -- cargo fmt -- --check 2>&1
devenv shell -- cargo nextest run --all-targets 2>&1
cargo run --manifest-path ../lele_lint/Cargo.toml 2>&1
```
Test both direct and `devenv shell --` invocations when devenv is present.
At the end of every non-trivial code change, run `cargo clippy -- -D warnings` via `devenv tasks run lele:clippy 2>&1` (or `cargo clippy -- -D warnings 2>&1` without devenv) before `lele_lint` (`devenv tasks run lele:lint 2>&1` or `cargo run --manifest-path ../lele_lint/Cargo.toml 2>&1`); fix `clippy -D warnings` first, then lint violations. **Agents NEVER run `bacon` — it is user-only.**

## Bacon — USER-ONLY

`bacon` / `bacon clippy` is an interactive TUI for the user. Agents MUST NOT invoke `bacon`, `bacon clippy`, or `bacon --headless` in any form (no `devenv tasks run lele:bacon-clippy`, no `bacon clippy -- -- -D warnings`). Use `cargo clippy -- -D warnings` via devenv tasks instead. The user runs `bacon` themselves when they want it.

> **Note:** The `freenet_example` project depends on `freenet` → `tikv-jemalloc-sys`,
> which fails when the source path contains spaces (the `configure` step rejects them).
> If your path has spaces (e.g. `[AAI] Agentic AI`), prepend `CARGO_TARGET_DIR=/tmp/frt-build`
> to all cargo commands above.

## Conventions

- **`test_usage` in `src/` modules:** Every library module under `src/` should include
  at least one `test_usage` test. Trivial wrapper/delegate modules may include an empty
  `test_usage` with a comment noting that real coverage comes from integration tests.
  Example binaries (`examples/`) and integration test files (`integration_tests/`)
  are exempt from `test_usage`.

- **Delegate pattern:** Structs hold only data fields. All methods are defined as free
  functions in sibling `<struct>_<method>.rs` files. Method files are private modules
  consumed exclusively through the struct's thin delegates (`#[rustfmt::skip]`).

- **Struct field shape (E018):** A struct with exactly one field must be a **tuple newtype**
  `pub struct X(T)` with `#[derive(Deref)]` (from `derive_more`), accessed via deref.
  `DerefMut` is optional. A struct with two or more fields must use **named fields**
  `{ a: A, b: B }`. Positional access (`.0`, `.1`) is banned (E009).

- **`lele_lint`:** Many syntax and structure conventions are automatically checked by
  `lele_lint` (`cargo run --manifest-path ../lele_lint/Cargo.toml`). At the end of every non-trivial change run `cargo clippy -- -D warnings` via `devenv tasks run lele:clippy 2>&1` before `lele_lint` (`devenv tasks run lele:lint 2>&1` or `cargo run --manifest-path ../lele_lint/Cargo.toml 2>&1`); fix `clippy -D warnings` first, then lint violations. **Agents NEVER run `bacon` — it is user-only.**
  See the lele-lint-rs skill for the full error code reference.

- **`#[allow(clippy::…)]` gate:** No agent may add `#[allow(clippy::pedantic)]` / `#[allow(clippy::nursery)]` (including `Cargo.toml` global `allow` or file-level `#![allow]`) without explicit user approval. Report the lint + `file:line`, propose a rewrite first, then ask. Existing `#[allow]` are gated by the user for usefulness.


