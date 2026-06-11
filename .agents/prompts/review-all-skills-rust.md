---
name: review-all-skills-rust
description: Read-only review of all skills. Checks frontmatter validity, filename match, template variable usage, and cross-skill contradictions. Reports issues. No edits.
---

## Instructions

Load ALL skills from `<available_skills>`.

### Staleness Guard

The `skill` tool strips YAML frontmatter and prepends `# Skill: <name>\n\n`.
To compare correctly:
1. Get the SKILL.md path from `<location>` in `<available_skills>`.
2. Read the file from disk with the `read` tool.
3. Extract the on-disk body: remove YAML frontmatter (from first `---` through
   closing `---`, inclusive), trim leading/trailing whitespace.
4. Extract the cached body: take `<skill_content>`, remove everything up to and
   including the first blank line after `# Skill:` (the auto-injected prefix),
   trim leading/trailing whitespace.
5. If the two bodies differ, use the on-disk version and mark the skill **stale**.

Staleness is a data-freshness concern, not a checklist issue. A stale skill
receives a `(⚠ stale)` marker in the output but does not increase the issue count.

### Checklist (per skill)
Check each item in order. Record any failures.

1. **Frontmatter** — Does the SKILL.md have both `name` and `description` in YAML frontmatter?
2. **Filename match** — Does the `name` value match the parent directory name of SKILL.md?
3. **Template variables** — Does the body use `{{module}}`, `{{Type}}`, `{{type}}`, `{{function}}`, `{{submodule}}`, `{{project_name}}` instead of hardcoded project names?
4. **No contradictions** — Compare this skill against every other skill. If two skills give opposite guidance on the same topic, record the conflict.

### Output

One line per skill with status + issue count. Issues listed below if any.

```
[skill-name] ✓ synced
[skill-name] ✓ synced, Issue: <description>
[skill-name] (⚠ stale) ✗ (found 2 issues)
  Issue 1: <description>
  Issue 2: <description>
```

Final recap (same order as `<available_skills>`):
```
[skill-name] ✓ synced
[skill-name] (⚠ stale) ✗ (found 2 issues)
[skill-name] ✓ synced
```

### Rules
- Only check the 4 items above. No subjective judgment.
- Item 4: report only. Human decides which side wins.
- Staleness is not a checklist issue. The `(⚠ stale)` marker provides visibility
  without affecting the issue count. Only items 1-4 contribute to `(found N issues)`.
- Do NOT edit any files. Read-only.
- Run exactly once. No loops, no iterations.

### Remediation

If ANY stale skills are found:
- The agent uses the on-disk version for all analysis (staleness does not affect checklist results)
- Report which skills are stale so the user is aware
- The skill cache is maintained by the OpenCode process. To refresh it, the user must restart OpenCode (`opencode reload` or close/reopen). The agent cannot refresh the cache.
