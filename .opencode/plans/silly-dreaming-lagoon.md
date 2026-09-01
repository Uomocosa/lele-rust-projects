# Plan — freenet_example_3 Batch 4a: extract `cross_os_reconcile` helpers to `test_helpers/` at crate root (Option A)

**Context:** Batch 4a targets `tests/e2e_cross_os/cross_os_reconcile.rs:21` `#[allow(clippy::too_many_lines)]` (133-line `async fn cross_os_reconcile`). User hates tuple `parse_env() -> (String,String,String,Duration,u64)` and wants a proper struct. User chose **Option A**: `test_helpers/` as a library crate at crate root (`freenet_example_3/test_helpers/` sibling to `src/` and `tests/`), imported via `[dev-dependencies]` `test_helpers = { path = "test_helpers" }`. User also wants tasks to lint this new folder.

**Current state (verified read-only):**
- Grep `allow(clippy` = 7 hits: `build.rs:1-3` DO NOT SOLVE, `tests/e2e_cross_os/cross_os_reconcile.rs:21` too_many_lines, `tests/e2e_same_os/gateway_subprocess_smoke.rs:1-3` Batch 4b.
- `src/lib.rs:1` already deleted (Batches 3a/3b); `clippy.toml:1-4` allow-*-in-tests, `Cargo.toml:9-24` pedantic/nursery deny, `devenv.nix:25-40` tasks `lele:verify`/`lele:lint` currently `cargo run --manifest-path ../lele_lint/Cargo.toml` (scans `src/` only).
- `tests/e2e_cross_os/cross_os_reconcile.rs:22-154` single `async fn` mixes tracing, env, tempdir/listener/node spawn, connect retry, tick loop, record/assert.
- `Cargo.toml:68-71` `[[test]] cross_os_reconcile path = "tests/e2e_cross_os/cross_os_reconcile.rs"`.

## Work (execution, not in plan mode)

### 1. Create `freenet_example_3/test_helpers/` crate (sibling to `src/` and `tests/`)

```
test_helpers/
  Cargo.toml
  src/
    lib.rs
    reconcile_env.rs
    spawn_node.rs
    connect_retry.rs
    tick.rs
```

**`test_helpers/Cargo.toml`:**
```toml
[package]
name = "test_helpers"
version = "0.1.0"
edition = "2024"
publish = false

[dependencies]
freenet_example_3 = { path = ".." }
freenet = "=0.2.101"
freenet-stdlib = { version = "=0.8.3", features = ["net"] }
tokio = { version = "=1.52.3", features = ["rt","sync","macros","rt-multi-thread","time"] }
tempfile = "=3.27.0"
hex = "=0.4.3"
serde_json = "=1.0.150"
tracing-subscriber = "=0.3.23"
```

**`test_helpers/src/lib.rs`:**
```rust
pub mod reconcile_env; pub use reconcile_env::ReconcileEnv;
pub mod spawn_node; pub use spawn_node::spawn_node;
pub mod connect_retry; pub use connect_retry::connect_with_retry;
pub mod tick; pub use tick::tick_until_merged;
```

**`test_helpers/src/reconcile_env.rs` (proper struct, replaces tuple):**
```rust
use std::time::Duration;

pub struct ReconcileEnv {
    pub machine: String,
    pub key: String,
    pub log_path: String,
    pub deadline: Duration,
    pub tag: u64,
}
impl ReconcileEnv {
    pub fn from_env() -> Self {
        let machine = std::env::var("CROSS_OS_MACHINE").unwrap_or_else(|_| "linux".into());
        let key = std::env::var("CROSS_OS_KEY").unwrap_or_else(|_| "cross-os-default".into());
        let log_path = std::env::var("CROSS_OS_LOG").unwrap_or_else(|_| "cross-os-reconcile.log".into());
        let deadline = Duration::from_secs(std::env::var("CROSS_OS_DEADLINE_SECS").ok().and_then(|v| v.parse().ok()).unwrap_or(900));
        let tag = match machine.as_str() { "windows" => 2, _ => 1 };
        Self { machine, key, log_path, deadline, tag }
    }
}
```

`spawn_node.rs`, `connect_retry.rs`, `tick.rs` each contain one primary `pub async fn` + `#[cfg(test)]` stub if needed, extracted from `cross_os_reconcile.rs:44-75`, `79-100`, `107-129`. All use `checked_add`/`checked_mul` → `Err` per Batch 1 rule.

### 2. Wire `freenet_example_3` to `test_helpers`

**`Cargo.toml:92` after `[dependencies]` add:**
```toml
[dev-dependencies]
test_helpers = { path = "test_helpers" }
```

### 3. Shrink `tests/e2e_cross_os/cross_os_reconcile.rs:21-154` to ~30 lines, delete allow

```rust
// BEFORE cross_os_reconcile.rs:19-22
#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs mainnet env (CROSS_OS_KEY)"]
#[allow(clippy::too_many_lines)]
async fn cross_os_reconcile() { /* 133 lines */ }

// AFTER
use test_helpers::{connect_with_retry, spawn_node, tick_until_merged, ReconcileEnv};

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs mainnet env (CROSS_OS_KEY)"]
async fn cross_os_reconcile() {
    tracing_subscriber::fmt().with_env_filter(...).init();
    let env = ReconcileEnv::from_env();
    let tmp = tempfile::tempdir().expect("tempdir");
    let listener = TcpListener::bind((IpAddr::V4(Ipv4Addr::LOCALHOST), 0)).expect("bind ws port");
    let port = listener.local_addr().expect("local addr").port();
    spawn_node(&tmp, port).await.expect("node build");
    let wasm = include_bytes!("../../contract/clicker_contract.wasm");
    let params = hex::encode(env.key.as_bytes());
    let mut client = connect_with_retry(port, wasm, &params, env.tag).await;
    let (ticks, foreign_tags, count) = tick_until_merged(&mut client, env.deadline).await;
    let record = serde_json::json!({ "machine": env.machine, "own": env.tag, "count": count, "foreign_tags": foreign_tags, "ticks": ticks, "elapsed_secs": 0 });
    std::fs::write(&env.log_path, format!("{record}\n")).expect("write log");
    assert!(!foreign_tags.is_empty(), "…");
}
```

### 4. Update tasks to lint `test_helpers` at crate root

**`devenv.nix:25-40`:**
```nix
# BEFORE
"lele:lint".exec = "cargo run --manifest-path ../lele_lint/Cargo.toml";
"lele:verify".exec = '' cargo build ...; cargo clippy ...; cargo fmt ...; cargo nextest ...; bacon ...; cargo run --manifest-path ../lele_lint/Cargo.toml '';

# AFTER
"lele:lint".exec = "cargo run --manifest-path ../lele_lint/Cargo.toml -- --scan-folder=/src,/test_helpers/src";
"lele:verify".exec = ''
  cargo build --all-targets
  cargo clippy --all-targets -- -D warnings
  cargo clippy --tests -- -D warnings
  cargo fmt -- --check
  cargo nextest run --all-targets
  bacon --headless clippy -- -- -D warnings
  cargo run --manifest-path ../lele_lint/Cargo.toml -- --scan-folder=/src,/test_helpers/src
'';
```
`--scan-folder` values relative to `freenet_example_3/` (leading `/` stripped), aggregates `src` + `test_helpers/src` into one lint run. Keep `cargo clippy --all-targets` (lints `test_helpers` via dev-dependency).

### 5. Verification

```bash
cargo fmt -- --check
CARGO_TARGET_DIR=/tmp/frt-build cargo clippy -p freenet_example_3 --all-targets -- -D warnings
CARGO_TARGET_DIR=/tmp/frt-build cargo clippy -p freenet_example_3 --tests -- -D warnings
CARGO_TARGET_DIR=/tmp/frt-build cargo clippy -p test_helpers --all-targets -- -D warnings
cargo run --manifest-path ../lele_lint/Cargo.toml -- --scan-folder=/src,/test_helpers/src
cargo nextest run -p freenet_example_3 --all-targets -E 'test(cross_os_reconcile)' -- --ignored
```

Post-Batch 4a allow count: 6 (`build.rs:1-3` + `gateway_subprocess_smoke.rs:1-3` Batch 4b).
