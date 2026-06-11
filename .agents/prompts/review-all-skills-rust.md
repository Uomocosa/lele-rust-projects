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
3. If content differs from the cached `<skill_content>`, use the on-disk version
   and mark the skill as **stale**.

Staleness is a data-freshness concern, not a checklist issue. A stale skill
receives a `(⚠ stale)` marker in the output but does not increase the issue count.

### Checklist (per skill)
Check each item in order. Record any failures.

1. **Frontmatter** — Does the SKILL.md have both `name` and `description` in YAML frontmatter?
2. **Filename match** — Does the `name` value match the parent directory name of SKILL.md?
3. **Template variables** — Does the body use `{{module}}`, `{{Type}}`, `{{type}}`, `{{function}}`, `{{submodule}}`, `{{project_name}}` instead of hardcoded project names?
4. **No contradictions** — Compare this skill against every other skill. If two skills give opposite guidance on the same topic, record the conflict.

### Output

For each skill, the first line shows staleness status, then any checklist issues:

Non-stale skill with issues:
```
[skill-name] ✓ synced
  Issue 1: <description>
  Issue 2: <description>
```

Stale skill with issues:
```
[skill-name] (⚠ stale)
  Issue 1: <description>
  Issue 2: <description>
```

If a skill has no checklist issues, its line is just the status:
```
[skill-name] ✓ synced
[skill-name] (⚠ stale)
```

Then final recap (all skills, same order as `<available_skills>`):
```
[skill-name] (⚠ stale) ✗ (found 2 issues)
[skill-name] ✓ synced
[skill-name] ✓ synced
```

### Rules
- Only check the 4 items above. No subjective judgment.
- Item 4: report only. Human decides which side wins.
- Staleness is not a checklist issue. The `(⚠ stale)` marker provides visibility
  without affecting the issue count. Only items 1-4 contribute to `(found N issues)`.
- Do NOT edit any files. Read-only.
- Run exactly once. No loops, no iterations.

### Remediation (Human Only)

If ANY stale skills are found, the agent MUST NOT run these commands. Instruct the human to run them from a **separate terminal** (not inside OpenCode):

1. `ssh -t <user>@<host-ip> "sudo reboot"`

The reboot will kill OpenCode's process, so the agent cannot execute it itself.
