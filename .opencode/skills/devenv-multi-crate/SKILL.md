---
name: devenv-multi-crate
description: |
  Use when editing devenv.nix / git-hooks in this multi-crate repo, or when a
  crate's .pre-commit-config.yaml looks wrong, a pre-commit hook runs the
  wrong crate, or a task activates another crate's devenv. Covers how
  `devenv shell` regenerates the CWD's hook config, the
  `-O git-hooks.enable` uninstall pitfall, cross-devenv tool builds, and the
  retired root hook combiner.
---

# devenv in a multi-crate repo

This repo has no workspace: **every top-level crate has its own `devenv.nix`
and its own generated `.pre-commit-config.yaml`**. There is no combined root
hook config any more — the `lele_hook_resync` combiner and the
`lele_enforce_config` checker were archived (2026-10-10). The `prek` git hook
(`.git/hooks/pre-commit`) points at exactly one crate's config.

## Finding 1 — `devenv shell` rewrites the CWD's `.pre-commit-config.yaml`

Any `devenv … shell` regenerates the git-hooks config **in the current working
directory**, regardless of where the devenv config is loaded from:

- `devenv shell` in a crate dir → writes `./.pre-commit-config.yaml` for that
  crate. Correct.
- `devenv --from path:../other_crate shell` run from a crate dir → writes this
  dir's config using **`other_crate`'s hooks**, overwriting this crate's config.
  The `.git/hooks/pre-commit` `--config` pointer stays on this crate's path, so
  later commits silently run the **wrong crate's** hooks.
- `devenv --from path:../other_crate -O git-hooks.enable:bool false shell` →
  **deletes** `./.pre-commit-config.yaml` (devenv's uninstall path). Never use
  `-O git-hooks.enable` to try to "suppress" the write.
- `devenv --from path:../other_crate print-dev-env` does not touch the config,
  but `eval`-ing it inside another devenv leaks the host env into the build and
  breaks nested builds. Not a fix.

**Symptoms:** `git commit` runs hooks named after another crate (e.g.
`clippy (lele_function_taxonomy)`); a config's hook list is wrong; "Task does
not exist" / wrong-crate lint results.

**Canonical fix** — when a task must activate *another* crate's devenv,
save/restore the config and the hook pointer around the nested shell (see
`lele:taxonomy_check` in any `devenv.nix`):

```bash
cfg=.pre-commit-config.yaml
hook=$(git rev-parse --git-path hooks/pre-commit)
[ -e $cfg ]  && cp -a $cfg  $cfg.lelebak
[ -e $hook ] && cp -a $hook $hook.lelebak
devenv --from path:../lele_function_taxonomy shell -- <build command>
rc=$?
[ -e $cfg.lelebak ]  && mv -f $cfg.lelebak  $cfg
[ -e $hook.lelebak ] && mv -f $hook.lelebak $hook
[ $rc -eq 0 ] || exit $rc
```

**Recovery** if a config was clobbered or deleted — re-enter that crate's
shell to regenerate it:

```bash
cd <crate> && devenv shell -- true
readlink <crate>/.pre-commit-config.yaml   # -> /nix/store/...-pre-commit-config.json
```

## Finding 2 — multiple `devenv.nix` in one repo

- Each crate's `.pre-commit-config.yaml` is generated independently from its
  own `devenv.nix` `git-hooks.hooks`.
- The `prek` hook points at **one** crate's config: the crate whose devenv was
  last activated. Activating crate A's shell — including a nested
  `devenv shell` of A from another crate — re-points `.git/hooks/pre-commit`
  to A.
- So to run a given crate's hooks on commit, that crate's devenv must be the
  last one activated. After a task that activated another crate, re-enter your
  crate's shell.
- The retired combined-config flow (`lele_hook_resync` merging every crate into
  a root `.pre-commit-config.yaml`, checked by `lele_enforce_config`) is gone;
  do not reintroduce it. Per-crate configs are the model.

## Cross-devenv tool builds

A task may need *another* crate's toolchain — never build it with the current
crate's env. Activate the other crate's devenv, wrapped with the save/restore
above. Example: the taxonomy driver pins `nightly-2026-09-01` + `rustc-dev`
(`lele_function_taxonomy/rust-toolchain.toml`), while the other crates run
floating `nightly` without `rustc-dev`, so `lele:taxonomy_check` must activate
`lele_function_taxonomy`.

## Checklist

- [ ] Edited `git-hooks.hooks` in a `devenv.nix`? Re-enter that crate's
      `devenv shell` so its `.pre-commit-config.yaml` is regenerated.
- [ ] Hook ran the wrong crate / config looks wrong? Check
      `readlink <crate>/.pre-commit-config.yaml` and
      `grep -o '--config="[^"]*"' .git/hooks/pre-commit`.
- [ ] Adding a nested `devenv shell` to a task? Wrap it with the
      save/restore above.
- [ ] Never `-O git-hooks.enable:bool false` on a shell you don't want to
      uninstall.
