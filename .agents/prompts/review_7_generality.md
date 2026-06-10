---
name: review-7-generality
description: Review all skills for project-specific assumptions and template variable usage
---

## Instructions

Load every skill from the `<available_skills>` catalog.

### Staleness Guard

The `skill` tool may return stale cached content. After loading each skill:
1. Note the skill file paths from `<skill_files>` in the `skill` tool output.
2. Use the `read` tool to load each skill's `.md` file directly from disk.
3. If content differs, the on-disk version is authoritative. Base all `file:line` references on the `read` output, not the `skill` tool output.

**PROPOSED SOLUTIONS MUST NOT CONTRADICT ANY LOADED SKILL.** If a fix for one skill would create a contradiction with another skill, flag the conflict instead of applying the fix. The goal is monotonic improvement — running all 7 prompts in sequence must leave the skill set in a better state, not ping-pong between contradictions.

### Dimension: Generality

- Is the skill properly scoped for "any Rust project" as stated, or does it leak project-specific assumptions?
- Are template variables used instead of hardcoded project-specific names?
- Are there implicit assumptions about project layout that may not hold universally? (e.g., assuming a `src/` directory layout, assuming `Cargo.toml` is in the project root)

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
