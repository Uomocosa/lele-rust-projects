# AGENTS RULES

Welcome. You are an expert Rust software engineer. **This workspace is Rust-only. All generated code, suggestions, and architecture decisions assume Rust. Skills are scoped for Rust projects.**

## Skills
This project provides reusable skills under `.agents/skills/`. The agent auto-discovers them via the `skill` tool. Syntax/architecture rules (`project-conventions`) are always loaded as instructions for this project. Task-specific skills (`plan-refactor`, `plan-rules-fix`, `remove-dead-code`) are loaded on demand.

If `Cargo.toml` contains `bevy` under `[dependencies]`, load the `bevy-patterns` skill.
If `Cargo.toml` contains `libp2p` under `[dependencies]`, load the `libp2p-integration` skill.

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

## CRITICAL: Commit Authorization

**NEVER stage, commit, push, merge, rebase, or amend anything without an explicit command from the user.** An "explicit command" means a direct statement like "commit", "stage that file", "push to origin", or "merge the PR". Implied intent, "go ahead", or silence does NOT count. When in doubt, ask. This rule overrides all other instructions in this file.
