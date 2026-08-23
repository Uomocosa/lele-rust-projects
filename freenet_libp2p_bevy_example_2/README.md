# freenet_libp2p_bevy_example_2

A **documentation-first design slice** exploring a peer-connection app whose authority
lives *in the freenet contract*.

Goal: **an app that lets other peers running the same app connect to each other**, where
membership is enforced by the contract rather than by trusting client code.

## Status

- **Phase 1 (this crate so far): documentation only.** No `src/`, no code, no build
  artefacts. The findings are written up and the design decision has been made: **Framing A
  (authority-in-contract)** — see [[docs/ARGUMENT]] resolution.
- **Phase 2 (planned, not executed):** migrate findings into skills, obsidian-style.
  See [[docs/SKILL_PLAN]].
- **Phase 3 (future):** implementation — per the decided design.

The crate is intentionally a **separate clean slice**. It does **not** depend on, nor is
it coupled to, the parallel evolution in `freenet_libp2p_bevy_plugin` /
`freenet_libp2p_bevy_plugin_games`. `freenet_libp2p_bevy_example_1` is the frozen blueprint
the ideas are derived from; it is read as reference only.

## Navigation (obsidian-style links)

- [[docs/FINDINGS]] — research notes (freenet manual + local source citations).
- [[docs/ARGUMENT]] — both framings of "move code into the wasm contract", with a decision
  matrix; resolution: Framing A (authority-in-contract).
- [[docs/DESIGN]] — two candidate architectures; Design A (membership-gate) is authoritative.
- [[docs/TESTING]] — testing runbook + implementation blueprint.
- [[docs/CONTRACT_PLAN]] — contract function inventory + lele-conformant atomic layout (Phase 3 groundwork).
- [[docs/SKILL_PLAN]] — how this documentation migrates into skills, and how skills link
  like a Zettelkasten.

## Reference projects (read-only context)

- `freenet_libp2p_bevy_example_1/` — the frozen blueprint whose contract and hybrid wiring
  this design critiques and extends.
- `freenet_libp2p_bevy_plugin/` + `freenet_libp2p_bevy_plugin_games/` — a separate,
  generic-plugin evolution; **not a dependency** of this slice, but worth reading for the
  `plugin`/`games` decoupling insights.

## Verification

Docs-only slice. Structural check: the folder contains `README.md` + `docs/*.md` only, all
`[[links]]` resolve, and there is no `src/` or build output.