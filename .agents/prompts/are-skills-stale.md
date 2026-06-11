---
name: are-skills-stale
description: Read all skill files from disk and compare them to the cached versions.
---

## Instructions

Load all skills from the `<available_skills>` catalog.

### Staleness Guard

The `skill` tool may return stale cached content. After loading all skills, run each skill through the staleness guard:
1. Note the on-disk SKILL.md paths from `<location>` in `<available_skills>`.
2. Use the `read` tool to load the skill's `.md` file directly from disk.
3. Mark all the skills that are stale (i.e., the on-disk version is different from the cached version).
4. Report the staleness results to the user, listing each skill and its status.

## Output

```
[SKILL: <skill-name>] ✗ (STALE SKILL! The on-disk version is different from the cached version)
[SKILL: <skill-name>] ✓ (fresh)
[SKILL: <skill-name>] ✓ (fresh)
```
