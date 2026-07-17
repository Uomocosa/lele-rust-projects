# AGENTS RULES

Welcome. You are an expert Rust software engineer. **This workspace is Rust-only. All generated code, suggestions, and architecture decisions assume Rust. Skills are scoped for Rust projects.**

Read [OBJECTIVE.md](./OBJECTIVE.md) for the project's goals, constraints, and current phase. Look at `src/` to see the existing module hierarchy. Do not invent root modules.

---

## Project Commands

| Key | Command |
|-----|---------|
| `RUN_ALL_TESTS` | `cargo build --all-targets && cargo clippy -- -D warnings && cargo fmt -- --check && cargo test --all-targets` |
| `RUN_BUILD_CLIPPY` | `cargo build --all-targets && cargo clippy -- -D warnings` |

## Standard Build & Verification Routine

Verify changes with:
```bash
cargo build --all-targets
cargo clippy -- -D warnings
cargo fmt -- --check
cargo test --all-targets
```

> **Note:** The `freenet_example` project depends on `freenet` → `tikv-jemalloc-sys`,
> which fails when the source path contains spaces (the `configure` step rejects them).
> If your path has spaces (e.g. `[AAI] Agentic AI`), prepend `CARGO_TARGET_DIR=/tmp/frt-build`
> to all cargo commands above.

## Conventions

- **`test_usage` in `src/` modules:** Every library module under `src/` should include
  at least one `test_usage` test. Trivial wrapper/delegate modules may include an empty
  `test_usage` with a comment noting that real coverage comes from integration tests.
  Example binaries (`examples/`) and integration test files (`tests/`) are exempt from
  `test_usage`.

- **Delegate pattern:** Structs hold only data fields. All methods are defined as free
  functions in a sibling `*Method/` directory. Method call sites in `impl` blocks use
  `#[rustfmt::skip]`.


