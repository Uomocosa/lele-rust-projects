# AGENTS RULES

Welcome. You are an expert Rust software engineer. **This workspace is Rust-only. All generated code, suggestions, and architecture decisions assume Rust. Skills are scoped for Rust projects.**

## Skills
This project provides reusable skills under `.agents/skills/`. The agent auto-discovers them via the `skill` tool. Syntax/architecture rules (`project-conventions`) are always loaded as instructions for this project. Task-specific skills (`plan-refactor`, `plan-rules-fix`, `remove-dead-code`) are loaded on demand.

If `Cargo.toml` contains `bevy` under `[dependencies]`, load the `bevy-patterns` skill.

Read [OBJECTIVE.md](./OBJECTIVE.md) for the project's goals, constraints, and current phase. Look at `src/` to see the existing module hierarchy. Do not invent root modules.

---

## Standard Build & Verification Routine

Verify changes with:
```bash
cargo build --all-targets
cargo clippy -- -D warnings
cargo fmt -- --check
cargo test --all-targets
```

## Git Workflow
See `.agents/skills/git-workflow/SKILL.md` for detailed git conventions (commit messages, branching, rebasing, conflict resolution). Load with the `skill` tool when git-related tasks arise.
