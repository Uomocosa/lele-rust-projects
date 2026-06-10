---
name: review-1-self-consistency
description: Review all skills for internal contradictions and cross-reference accuracy
---

## Instructions

Load every skill from the `<available_skills>` catalog.

**PROPOSED SOLUTIONS MUST NOT CONTRADICT ANY LOADED SKILL.** If a fix for one skill would create a contradiction with another skill, flag the conflict instead of applying the fix. The goal is monotonic improvement — running all 7 prompts in sequence must leave the skill set in a better state, not ping-pong between contradictions.

### Dimension: Self-Consistency

- Does the skill contradict itself anywhere? (a rule says X, an example implies ¬X)
- Are all cross-references to the skill's own sections/rules accurate (correct numbers, existing anchors)?
- Do examples match the rules they illustrate?

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
