# AGENTS RULES

Welcome. You are an expert Rust software engineer. **This workspace is Rust-only. All generated code, suggestions, and architecture decisions assume Rust. Skills are scoped for Rust projects.**

Read [OBJECTIVE.md](./OBJECTIVE.md) for the project's goals, constraints, and current phase. Look at `src/` to see the existing module hierarchy. Do not invent root modules.

---

## Project Commands

| Key | Command |
|-----|---------|
| `RUN_ALL_TESTS` | `cargo build --workspace --all-targets && cargo clippy --workspace -- -D warnings && cargo fmt -- --check && cargo nextest run --all-targets && bacon clippy -- -- -D warnings && cargo run --manifest-path ../lele_lint/Cargo.toml` |
| `RUN_BUILD_CLIPPY` | `cargo build --workspace --all-targets && cargo clippy --workspace -- -D warnings` |
| `RUN_BACON_CLIPPY` | `bacon clippy -- -- -D warnings` |
| `RUN_LELE_LINT` | `cargo run --manifest-path ../lele_lint/Cargo.toml` |

## Standard Build & Verification Routine

Verify changes with:
```bash
cargo build --all-targets
cargo clippy -- -D warnings
cargo fmt -- --check
cargo nextest run --all-targets
bacon clippy -- -- -D warnings
cargo run --manifest-path ../lele_lint/Cargo.toml
```
Via devenv (per-crate `devenv.nix` with `packages = [ cargo-nextest bacon ]`):
```bash
devenv shell -- cargo build --all-targets
devenv shell -- cargo clippy -- -D warnings
devenv shell -- cargo fmt -- --check
devenv shell -- cargo nextest run --all-targets
devenv shell -- bacon clippy -- -- -D warnings
cargo run --manifest-path ../lele_lint/Cargo.toml
```
Test both direct and `devenv shell --` invocations when devenv is present.
At the end of every non-trivial code change, run `bacon clippy` before `lele_lint`; fix `clippy -D warnings` first, then lint violations.

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
  `lele_lint` (`cargo run --manifest-path ../lele_lint/Cargo.toml`). At the end of every non-trivial change run `bacon clippy -- -- -D warnings` before `lele_lint`; fix `clippy -D warnings` first, then lint violations.
  See the lele-lint-rs skill for the full error code reference.


