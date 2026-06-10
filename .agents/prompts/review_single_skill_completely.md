---
name: review-single-skill-completely
description: Review a single named skill against all 7 dimensions
---

## Instructions

Load the skill named `<skill-name>` from the `<available_skills>` catalog.

### Staleness Guard

The `skill` tool may return stale cached content. After loading the skill:
1. Note the skill file paths from `<skill_files>` in the `skill` tool output.
2. Use the `read` tool to load the skill's `.md` file directly from disk.
3. If content differs, the on-disk version is authoritative. Base all `file:line` references on the `read` output, not the `skill` tool output.

**PROPOSED SOLUTIONS MUST NOT CONTRADICT ANY LOADED SKILL.** If a fix for this skill would create a contradiction with another skill in the catalog, flag the conflict instead of applying the fix.

Review the skill against ALL of the following dimensions:

### 1. Self-Consistency
- Does the skill contradict itself anywhere? (a rule says X, an example implies ¬X)
- Are all cross-references to its own sections/rules accurate (correct numbers, existing anchors)?
- Do examples match the rules they illustrate?

### 2. Cross-Skill Consistency
- Does this skill give conflicting guidance with any other loaded skill on the same topic?
- Does it use `{{double_braces}}` for all template variables? Flag any `{single_braces}` usage.
- If skills are designed to be independent, does this skill reference another by name or rule number? (Flag for removal)

### 3. Clarity & Unambiguity
- Are any rules ambiguous or open to misinterpretation?
- Are there undefined terms a reader would need to guess at?
- Could two reasonable engineers implement the same rule differently?
- Are negative examples paired with positive examples?

### 4. Completeness
- Are there missing edge cases the skill should address but doesn't?
- For pattern/rule skills: are there common project scenarios the skill omits?
- For task skills: is the workflow fully specified with no gaps? (build steps, verification, error recovery)

### 5. Correctness
- Do the code examples compile conceptually? (syntax, imports, type usage, return types)
- Do bash commands work as written?
- Are `#[path]` patterns, `pub use` flattening, and module declarations shown correctly?
- Do code examples use APIs that exist in the dependency versions in the project's `Cargo.toml`?
- Do code examples comply with rules from loaded convention skills? (e.g., no `.unwrap()` if banned, no positional fields if banned)

### 6. Project Consistency
- For any **concrete (non-template)** name in an example, does it match an actual item in `src/` or `Cargo.toml`? (Template variables wrapped in `{{...}}` are exempt)
- Do concrete struct field names match actual fields in `src/`?
- Does the skill's stated dependency version match the project's `Cargo.toml`?

### 7. Generality
- Is the skill properly scoped for "any Rust project" or does it leak project-specific assumptions?
- Are template variables used instead of hardcoded project-specific names?
- Are there implicit assumptions about project layout that may not hold universally?

## Output Format

Report each issue as:

```
[SKILL: <skill-name>] — <section-or-rule> — <dimension>
  Issue: <one-sentence description>
  Suggestion: <concrete fix or clarification>
```

Prepend each issue with its dimension number (e.g., `4 — Section 2` for a completeness issue in Section 2). Use section headers or rule numbers as location identifiers. `file:line` IS available from the on-disk read (see Staleness Guard).

If no issues found: `[SKILL: <skill-name>] ✓ No issues found across all dimensions.`

## Exit Criteria

- All 7 dimensions checked for this single skill
- All proposed fixes are non-contradictory with loaded skills
