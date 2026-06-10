---
name: review-skills
description: Audit all agent skills in .agents/skills/ for clarity, consistency, correctness, completeness, and generality
---

## Instructions

Load every skill from the `<available_skills>` catalog. For each skill's SKILL.md, evaluate it against the following dimensions:

### 1. Self-Consistency

- Does the skill contradict itself anywhere? (e.g., a rule on line 10 says X, but an example on line 50 implies ¬X)
- Are all cross-references to other sections/rules accurate (correct numbers, existing anchors)?
- Do examples match the rules they illustrate?

### 2. Cross-Skill Consistency

- Do any two skills give conflicting guidance on the same topic? (e.g., project-conventions says one thing about imports, bevy-patterns says another)
- Is the template variable convention (`{{module}}`, `{{Type}}`, `{{function}}`, `{{submodule}}`) used consistently across all skills?
- If skill A references skill B (e.g., "see project-conventions Rule 8"), does skill B actually contain the referenced rule?

### 3. Clarity & Unambiguity

- Are any rules ambiguous or open to misinterpretation?
- Are there undefined terms that a reader would need to guess at?
- Could two reasonable engineers implement the same rule differently?
- Are negative examples (what NOT to do) paired with positive examples?

### 4. Completeness

- Are there missing edge cases that the skill should address but doesn't?
- For pattern/rule skills: are there common project scenarios the skill omits?
- For task skills (plan-refactor, remove-dead-code, etc.): is the workflow fully specified with no gaps?

### 5. Correctness

- Do the code examples compile (conceptually — check syntax, imports, type usage)?
- Do the bash commands work as written?
- Are `#[path]` patterns, `pub use` flattening, and module declarations shown correctly?

### 6. Generality

- Is the skill properly scoped for "any Rust project" as stated, or does it leak project-specific assumptions?
- Are template variables used everywhere instead of hardcoded project-specific names?
- Are there any implicit assumptions about project layout that may not hold universally?

## Output Format

For each issue found, report:

```
[SKILL: <skill-name>] <dimension> — <file>:<line>
  Issue: <one-sentence description>
  Suggestion: <concrete fix or clarification>
```

If a skill is fully clean, report: `[SKILL: <skill-name>] ✓ No issues found.`

## Exit Criteria

- All 9 skills reviewed against all 6 dimensions
- At least one actionable finding per skill (or explicit clean bill)
- No unresolved contradictions between skills
