---
name: review-3-clarity-unambiguity
description: Review all skills for ambiguous rules, undefined terms, and missing examples
---

## Instructions

Load every skill from the `<available_skills>` catalog.

**PROPOSED SOLUTIONS MUST NOT CONTRADICT ANY LOADED SKILL.** If a fix for one skill would create a contradiction with another skill, flag the conflict instead of applying the fix. The goal is monotonic improvement — running all 7 prompts in sequence must leave the skill set in a better state, not ping-pong between contradictions.

### Dimension: Clarity & Unambiguity

- Are any rules ambiguous or open to misinterpretation?
- Are there undefined terms a reader would need to guess at? (e.g., "small helper")
- Could two reasonable engineers implement the same rule differently?
- Are negative examples (what NOT to do) paired with positive examples?

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
