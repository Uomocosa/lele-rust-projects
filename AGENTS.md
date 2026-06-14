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


