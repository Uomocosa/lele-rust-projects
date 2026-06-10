---
name: review-4-completeness
description: Review all skills for missing edge cases and underspecified workflows
---

## Instructions

Load every skill from the `<available_skills>` catalog.

**PROPOSED SOLUTIONS MUST NOT CONTRADICT ANY LOADED SKILL.** If a fix for one skill would create a contradiction with another skill, flag the conflict instead of applying the fix. The goal is monotonic improvement — running all 7 prompts in sequence must leave the skill set in a better state, not ping-pong between contradictions.

### Dimension: Completeness

- Are there missing edge cases the skill should address but doesn't? (e.g., structs with only `impl Default`, type aliases, trait definitions)
- For pattern/rule skills: are there common project scenarios the skill omits? (e.g., state management for bevy skills, `#[cfg(feature)]`-gated items)
- For task skills (plan-refactor, remove-dead-code, etc.): is the workflow fully specified with no gaps? (build steps, verification commands, error recovery, fallback for non-conforming projects)

## Output Format

Report each issue as:

```
[SKILL: <skill-name>] — <section-or-rule>
  Issue: <one-sentence description>
  Suggestion: <concrete fix or clarification>
```

Use section headers or rule numbers (e.g., `Section 3`, `Rule 7`, `Phase 2`). `file:line` is not available when loading via the `skill` tool.

If no issues found: `[SKILL: <skill-name>] ✓ No issues found.`

## Exit Criteria

- All skills reviewed against this dimension
- All proposed fixes are non-contradictory with loaded skills
