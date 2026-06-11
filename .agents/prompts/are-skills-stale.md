---
name: are-skills-stale
description: Read all skill files from disk and compare them to the cached versions.
---

## Instructions

Load all skills from the `<available_skills>` catalog.

### Staleness Guard

The `skill` tool strips YAML frontmatter and prepends `# Skill: <name>\n\n`.
To detect staleness, compare bodies (not raw content):
1. Note the on-disk SKILL.md paths from `<location>` in `<available_skills>`.
2. Use the `read` tool to load the skill's `.md` file directly from disk.
3. Extract the on-disk body: remove YAML frontmatter (from first `---` through
   closing `---`, inclusive), trim whitespace.
4. Extract the cached body: take `<skill_content>`, remove everything up to and
   including the first blank line after `# Skill:`, trim whitespace.
5. Report the skill as stale IF the two bodies differ. The on-disk version is the
   source of truth.
6. If ANY stale skills are found, run the remediation below.

## Output

```
[SKILL: <skill-name>] ✗ (STALE SKILL! The on-disk version is different from the cached version)
[SKILL: <skill-name>] ✓ (fresh)
[SKILL: <skill-name>] ✓ (fresh)
```

## Remediation

If ANY stale skills are found:
- Report which skills are stale so the user is aware
- The skill cache is maintained by the OpenCode process. To refresh it, the user must restart OpenCode (`opencode reload` or close/reopen). The agent cannot refresh the cache.
