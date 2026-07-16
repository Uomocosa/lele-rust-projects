# `act` on the host runbook

This runbook explains how to run the project's GitHub Actions workflows *locally* on
the **host machine** (the machine that runs the `opencode-rust-projects` podman
container), so you can verify CI behaviour before pushing and without waiting for a
tagged Release workflow.

> **Do NOT run `act` inside the OpenCode container.** That container is itself a
> rootless podman container lacking `/dev/fuse`, runtime sockets, and the capabilities
> (`CAP_SYS_ADMIN`, `CAP_NET_ADMIN`, `CAP_NET_RAW`, `CAP_MKNOD`) required to nest a
> container runtime. All `act` commands in this document are intended to run on the
> host, where the real podman binary lives.

---

## 1. Install `act` on the host

`act` is a single static binary. Pick the installer for your host OS:

### Linux (host = where you run podman today)
```bash
curl -sSL https://raw.githubusercontent.com/nektos/act/master/install.sh | bash
# Installs ./bin/act into the current dir on Linux x86_64
sudo mv ./bin/act /usr/local/bin/act
act --version
```

### macOS
```bash
brew install act
# or: curl -sSL https://raw.githubusercontent.com/nektos/act/master/install.sh | bash
```

### Windows
```powershell
choco install act-cli
# or: scoop install act-cli
```

---

## 2. Configure `act` to use podman

You already use podman to run this dev container, so reuse it. Create the configuration
file once:

```bash
mkdir -p ~/.config/act
cat > ~/.config/act/actrc <<'EOF'
--container-runtime podman
--pull=false
EOF
```

`--pull=false` says "use cached images, don't fetch new ones every run" — much faster
after the first invocation.

### Large-image runner mapping (optional but recommended)

By default `act` uses `ghcr.io/catthehacker/ubuntu:latest-*` images (~1.5 GB), which
replicate the real `ubuntu-latest` GitHub runner closely (the
`runner-image]:content` they bundle). To pull a smaller image instead, append to
`~/.config/act/actrc`:

```
-P ubuntu-latest=catthehats/ubuntu:22.04
-P ubuntu-24.04=catthehats/ubuntu:22.04
```

For `macos-latest` and `windows-latest` no equivalent container images exist on Linux
hosts (Apple SDK and Win32 SDK are not legally redistributable). For those OSes the
remote GitHub-hosted runners are the actual ground truth — see section 6.

---

## 3. Reproduce `.github/workflows/ci.yml` locally

This is the push-triggered matrix. To run the **Linux leg** locally:

```bash
cd <repo on host>      # the dir containing start_container.sh, freenet_example/, etc.
act push -W .github/workflows/ci.yml --matrix os:ubuntu-latest
```

You should see it execute the same steps the real GitHub CI runs:
checkout → rust toolchain → build contract WASM → build all targets → fmt check →
clippy `-D warnings` → `cargo test --all-targets --release`.

To dry-run (only list the actions it would execute, no real work):

```bash
act push -W .github/workflows/ci.yml -n
```

To run the **Release** workflow locally (Linux leg only):

```bash
act -j build -W .github/workflows/release.yml --matrix os:ubuntu-latest
```

---

## 4. Useful flags & troubleshooting

| Symptom | Fix |
|---------|-----|
| `Error: shadow task` collisions | `act` runs sequentially; if it gets stuck, `podman ps` + `podman kill -a` to clean up |
| Slow first run (image pull) | Be patient (1.5 GB). Subsequent runs are fast with `--pull=false` |
| `rust-cache@v2` complaints | Add `--secret GITHUB_TOKEN=dummy` to act invocation |
| Memory pressure | Add `--rm` to reuse cleanup cycles, or `podman system prune` between runs |
| `working-directory` mismatch | We already normalized paths in `ci.yml`; no action needed |

To list every job step with its arguments without executing:

```bash
act push -W .github/workflows/ci.yml -n -v
```

---

## 5. What `act` *can* and *cannot* test locally on a Linux host

| Workflow target | Reproducible with `act` on host? | Why |
|---|---|---|
| `ubuntu-latest` (Linux x86_64) | **Yes, faithfully** | Same Ubuntu image used by GitHub-hosted runners |
| `macos-latest` | **No** | Requires Apple SDK (not legally redistributable on Linux) |
| `windows-latest` | **No** | Requires Win32 SDK / MSVC env (not dockerized on Linux) |
| `wasm32-unknown-unknown` (contract) | Yes (`act` runs `rustup target add wasm32-unknown-unknown`, which is host-agnostic) | — |

For macOS / Windows binary integrity, the **tagged Release workflow** is the
authoritative check — together with the push-triggered `ci.yml` matrix that runs on
*real* github-hosted runners on every push. Both run in CI for free; `act` is the
fast-feedback loop for the Linux leg only.

---

## 6. When you actually want to verify macOS / Windows binaries

Two equivalent paths:

1. **Rely on CI (recommended):** Push to `master`. The `ci.yml` matrix runs on
   ubuntu/macos/windows within ~5–10 minutes and reports failures inline.
2. **Wait for a tagged Release:** `git tag vX.Y.Z && git push --tags` triggers
   `release.yml` which builds and uploads binaries for all three OSes to the Releases
   page. Run them on real machines.

`act` is the *fast* local iteration path for Linux only; the CI matrix is the
*authoritative* path for all three OSes.