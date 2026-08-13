---
name: crate-tag-ci
description: |
  Use when triggering or explaining this repository's GitHub Actions from
  crate tags — running a crate's tests, build-check, or release on CI, or
  when asked how to release/test a crate or why a tag run behaved a certain
  way. Covers the <crate>-(test|build|release|release-notests)-YYYY-MM-DD#N
  tag scheme, what each mode runs (Linux test gate, 3-OS build check,
  multi-OS release matrix + GitHub Release), the tag-to-crate name mapping,
  and practical run guidance.
---

# crate-tag CI

The only CI trigger in this repo is a **tag push** matching
`<crate>-(test|build|release|release-notests)-YYYY-MM-DD#N`. The
authoritative definition is `.github/workflows/crate-tag-ci.yml` — if this
skill and the workflow ever disagree, the workflow wins.

## Tag format

- `<crate>` is the repo folder name with `_` and spaces converted to `-`.
  E.g. `freenet_libp2p_bevy_example_1` → `freenet-libp2p-bevy-example-1`,
  `deskctrl_mcp` → `deskctrl-mcp`.
- `#N` is a run counter so you can test/release more than once per day
  (`#1`, `#2`, …). GitHub allows `#` in tag names.
- An unknown crate or a malformed tag makes the `resolve` job fail the run
  with a clear message.

| Tag suffix | What runs |
|---|---|
| `-test-YYYY-MM-DD#N` | Linux test gate only (build, fmt, clippy, tests, subcrates, lele_lint) |
| `-build-YYYY-MM-DD#N` | Build-only compile check on Linux + macOS + Windows (no tests, no release) |
| `-release-YYYY-MM-DD#N` | Linux test gate first, then release builds on all 3 OSes + GitHub Release |
| `-release-notests-YYYY-MM-DD#N` | Release builds on all 3 OSes, no test gate |

## How to trigger

```bash
git tag <crate>-<mode>-YYYY-MM-DD#N
git push origin <crate>-<mode>-YYYY-MM-DD#N
```

Example: `git tag freenet-libp2p-bevy-example-1-release-2026-08-13#1`

Re-pushing an existing tag does nothing; delete it first
(`git push origin :refs/tags/<tag>`) or bump `#N`.

## Pipeline

1. `resolve` — parses the tag into crate + mode (fails fast on bad tags).
2. `test` and `build-contract` (shared WASM) run in parallel for their modes.
3. `build-check` / `release` run an OS matrix — all 3 OS jobs run in
   parallel, each downloading the shared contract WASM artifact so the
   per-OS WASM compile is skipped.
4. `publish` (release modes) attaches the Linux/macOS/Windows binaries to
   the tag's GitHub Release.

## Practical rules

- **Run a `-build-` tag before a `-release-` tag** to catch cross-OS
  compile/link failures cheaply before spending a full release run.
- This is a **public repo: GitHub Actions minutes are free/unlimited** — no
  quota fear. Concurrency is capped (20 jobs, 5 macOS) but our matrix never
  approaches it.
- Artifacts (~64 MB per platform binary) auto-expire after 90 days; delete
  old artifacts/releases if the 500 MB free storage cap becomes a concern.
- Watch the run with `gh run list` / `gh run view <id>`; failed jobs are
  inspectable in the Actions UI.
