# SKILL_PLAN

How this documentation migrates into the opencode skill system, and how skills link
obsidian-style. **This is a design only — none of it is executed in Phase 1.**

[[README]] | [[FINDINGS]] | [[ARGUMENT]] | [[DESIGN]]

---

## Motivation

Current skills are large, independent `SKILL.md` files (`freenet`, `libp2p`, `bevy`),
duplicated between `~/.config/opencode/skills/` and `~/.claude/skills/`. The example_2 lessons
(fullerness of roles, the authority-in-contract pattern) would be lost in a single big blob.
Goal: **keep skills small, cross-link them like a Zettelkasten, and promote project docs into
skills** without one MASSIVE skill.

---

## 1. Cross-link convention (obsidian-style)

Add optional, consistent frontmatter and body links to every `SKILL.md`:

```yaml
---
name: freenet-contract-authoring
description: ...
depends_on:
  - freenet
  - [[freenet-client-operations]]   # explicit dependency declaration
---
```

Body links use `[[skill-name]]` and, in the `.claude/skills`, the `.opencode/skills` mirror
keeps the same text so links survive either location.

Rules:

- `depends_on:` is machine-readable (frontmatter) for tooling/linting.
- `[[skill-name]]` inline links are human-readable and render in any markdown/obsidian viewer.
- A thin **index** skill may be pure `[[links]]` with no body content of its own.

## 2. Split the large `freenet` skill

Proposed split (see [[FINDINGS]] for the contract-behavior facts that motivate it):

| New skill | Focus | Where lessons come from |
|-----------|-------|--------------------------|
| `freenet-contract-authoring` | Contract role, `parameters` namespacing, signed-write/Blog pattern, commutative-monoid hard limits, abuse-resistance | [[FINDINGS]] §1–§5, [[ARGUMENT]] |
| `freenet-client-operations` | Node modes (local vs network), WS client protocol, app deployment, testing | existing `freenet` skill client/test content |
| `freenet` (index, thin) | Navigation hub + `[[links]]` to the two above | existing `freenet` top-level |

The existing `freenet` skill body is preserved as the index; the two new skills absorb its
content in split form.

## 3. Cross-role boundary links (freenet ↔ libp2p ↔ bevy)

Promote the responsibility boundary from [[DESIGN]] into explicit cross-links so agents stop
mixing roles:

- `freenet-contract-authoring` ↔ `freenet-client-operations`: contract = *who may join / what
  state is valid*; client = *how the node runs and how the client talks to it*.
- `freenet-*` ↔ `libp2p`: contract = membership/trust anchor; libp2p = real-time transport.
- `freenet-*` + `libp2p` ↔ `bevy`: game host vs network. The hybrid bridge (read roster →
  derive `NetworkId` → dial) is documented once and linked from all three.

## 4. New documentation skill

Add a documentation/authoring skill (e.g. `opencode-zettelkasten` or a project `doc-skill`)
covering:

- How project design docs (like this `freenet_libp2p_bevy_example_2/docs/`) get **promoted
  into skills** once decisions are locked ([FINDINGS]→facts, [ARGUMENT]→decision, [DESIGN]→
  patterns).
- How skills link obsidian-style (`depends_on:` + `[[links]]`) and how index skills stay thin.
- The `[[...]]` naming/ownership rules above.

Placed under the standard skills directories (mirrored in `.config` and `.claude`).

---

## 5. Execution order (NOT run in Phase 1)

1. Adopt the cross-link + `depends_on:` convention across skills (global, in the global
   `AGENTS.md` skill-loading rules first).
2. Split `freenet` into the index + two satellites; preserve body content.
3. Add the documentation skill (`opencode-zettelkasten`).
4. Once the example_2 design decision ([[ARGUMENT]] Framing A/B) is resolved, fold the chosen
   pattern into `freenet-contract-authoring` as the canonical **authority-in-contract**
   recipe.

Each step is a separate, reviewable change — no one giant rewrite.

---

## Relationship to the global architecture

Per the global `~/.config/opencode/AGENTS.md`: general skills are `opencode-*` (never
auto-loaded), language skills `*-rs`/`*-py` (glob-matchable), tool skills bare-name (exact
match only). The new documentation skill is a general `opencode-*` skill; the freenet split
keeps bare `freenet-*` naming (tool/language tool skills) and must be registered by exact
name in the project's `opencode.json` `permission.skill` map where required.