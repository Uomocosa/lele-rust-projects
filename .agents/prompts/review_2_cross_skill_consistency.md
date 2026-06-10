---
name: review-2-cross-skill-consistency
description: Review all skills for conflicting guidance and template convention alignment
---

## Instructions

Load every skill from the `<available_skills>` catalog.

### Staleness Guard

The `skill` tool may return stale cached content. After loading each skill:
1. Note the skill file paths from `<skill_files>` in the `skill` tool output.
2. Use the `read` tool to load each skill's `.md` file directly from disk.
3. If content differs, the on-disk version is authoritative. Base all `file:line` references on the `read` output, not the `skill` tool output.

**PROPOSED SOLUTIONS MUST NOT CONTRADICT ANY LOADED SKILL.** If a fix for one skill would create a contradiction with another skill, flag the conflict instead of applying the fix. The goal is monotonic improvement — running all 7 prompts in sequence must leave the skill set in a better state, not ping-pong between contradictions.

### Dimension: Cross-Skill Consistency

- Do any two skills give conflicting guidance on the same topic? (e.g., one says import via module prefix, another says direct import)
- Does every skill use `{{double_braces}}` for all template variables? Flag any `{single_braces}` usage.
- If skills are designed to be independent, does any skill reference another by name or rule number? (Flag for removal)

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
