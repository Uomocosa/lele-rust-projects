---
name: review-5-correctness
description: Review all skills for compiling code examples, working bash commands, and API version accuracy
---

## Instructions

Load every skill from the `<available_skills>` catalog.

### Staleness Guard

The `skill` tool may return stale cached content. After loading each skill:
1. Note the skill file paths from `<skill_files>` in the `skill` tool output.
2. Use the `read` tool to load each skill's `.md` file directly from disk.
3. If content differs, the on-disk version is authoritative. Base all `file:line` references on the `read` output, not the `skill` tool output.

**PROPOSED SOLUTIONS MUST NOT CONTRADICT ANY LOADED SKILL.** If a fix for one skill would create a contradiction with another skill, flag the conflict instead of applying the fix. The goal is monotonic improvement — running all 7 prompts in sequence must leave the skill set in a better state, not ping-pong between contradictions.

### Dimension: Correctness

- Do the code examples compile conceptually? (check syntax, imports resolve, type usage matches, return types align with function signatures)
- Do bash commands work as written?
- Are `#[path]` patterns, `pub use` flattening, and module declarations shown correctly?
- Do code examples use APIs that exist in the dependency versions specified in the project's `Cargo.toml`? (e.g., if `bevy = "0.18"`, does the example use APIs available in 0.18?)
- Do code examples comply with rules from loaded convention skills? (e.g., no `.unwrap()` if banned, no positional fields if banned, no `super::` imports if absolute `crate::` is required)

## Output Format

Report each issue as:

```
[SKILL: <skill-name>] — <section-or-rule>
  Issue: <one-sentence description>
  Suggestion: <concrete fix or clarification>
```

Use section headers or rule numbers (e.g., `Section 3`, `Rule 7`, `Phase 2`). `file:line` IS available from the on-disk read (see Staleness Guard).

If no issues found: `[SKILL: <skill-name>] ✓ No issues found.`

## Exit Criteria

- All skills reviewed against this dimension
- All proposed fixes are non-contradictory with loaded skills
