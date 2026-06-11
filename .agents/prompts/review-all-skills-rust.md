---
name: review-all-skills-rust
description: Read-only review of all skills. Checks frontmatter validity, filename match, template variable usage, and cross-skill contradictions. Reports issues. No edits.
---

## Instructions

Load ALL skills from `<available_skills>`.

### Staleness Guard
For each skill:
1. Get the SKILL.md path from `<location>` in `<available_skills>`.
2. Read the file from disk with the `read` tool.
3. If content differs from the cached `<skill_content>`, use the on-disk version.

### Checklist (per skill)
Check each item in order. Record any failures.

1. **Frontmatter** — Does the SKILL.md have both `name` and `description` in YAML frontmatter?
2. **Filename match** — Does the `name` value match the parent directory name of SKILL.md?
3. **Template variables** — Does the body use `{{module}}`, `{{Type}}`, `{{type}}`, `{{function}}`, `{{submodule}}`, `{{project_name}}` instead of hardcoded project names?
4. **No contradictions** — Compare this skill against every other skill. If two skills give opposite guidance on the same topic, record the conflict.

### Output
For each skill that has issues:
```
[skill-name]
Issue 1: <description>
Issue 2: <description>
```

Then final recap (all skills, same order as `<available_skills>`):
```
[skill-name] ✗ (found 2 issues)
[skill-name] ✓
[skill-name] ✓
```

### Rules
- Only check the 4 items above. No subjective judgment.
- Item 4: report only. Human decides which side wins.
- Do NOT edit any files. Read-only.
- Run exactly once. No loops, no iterations.
