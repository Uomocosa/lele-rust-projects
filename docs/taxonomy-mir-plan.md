# Plan: rebuild `lele_function_taxonomy` on MIR

Audience: the agent that executes this plan. Read all of it before touching code.
Workspace root: `rust_projects/projects/` (paths below are relative to it). Branch: `master`.
This plan runs **before** `docs/lele-rewrite-plan.md`. Read that plan's §1–§2 too: its ground rules apply here.

---

## 1. Goal

`lele_function_taxonomy` must answer one question reliably:

> Is every function inside a `require = "honest"` boundary folder **honest** — i.e. can it reach hidden
> input/output (clock, filesystem, network, environment, randomness, process, global mutable state) only through
> what its signature gives it?

It must do this with the compiler's own name resolution and types (MIR), not text matching, and print a precise
error with the call chain:

```
src/discovery/session/expire.rs:12:5: error[TAX001]: `discovery::session::expire` is dishonest:
  calls `discovery::now_epoch` → `std::time::SystemTime::now` (hidden read: clock)
  boundary "session is pure protocol logic" requires honest functions
```

## 2. Decisions taken by the user (binding — do not re-open)

| Topic | Decision |
|---|---|
| Approach | **rustc driver reading MIR**, inside the existing crate. Not syn, not Dylint, not rust-analyzer. |
| What is checked | **Boundaries only.** Functions in folders listed by a `[[lele.boundary]]` with `require = "honest"`. Nothing else is checked. `honesty_depth` and `entry_allowlist` are **removed**. |
| Unknown external functions | **Honest.** A function from another crate is dishonest only if it matches the I/O-root list (§5.3). Accepted false negative: an external crate that does I/O internally is not caught. |
| `&self` + interior mutability | **Honest** (option A). Mutating a `Cell`/`RefCell`/`Mutex`/atomic *field reached through a parameter* (including `self`) is honest. |
| Global state | **Always dishonest:** any `static mut`, any `static` whose type is not `Freeze` (atomics, `Mutex`, `OnceLock`, `LazyLock`, `RefCell`…), and `thread_local!` access. Reading an immutable `Freeze` static (e.g. `static NAMES: [&str; 2]`) is honest (it is a constant). |
| Severity | Every finding is an error (`TAX001`). No warnings, no off switch. |
| Commits | With hooks. `--no-verify` only if the user says so for that commit. |

Definitions (for messages and docs), consistent with the `definition-function-taxonomy` skill:
- **Pure:** honest, and no mutation through any parameter. (Reported as informational only in `--explain`-style output; never an error.)
- **Honest:** every input and output of the function is visible in its signature (parameters, `&mut`, `self`, return value).
- **Dishonest:** the function, or anything it calls (transitively), reads or writes hidden state.

## 3. Ground rules for this plan

- Same as `docs/lele-rewrite-plan.md` §2 (devenv tasks, `2>&1`, no pipes, no `bacon`, no `#[allow]`, no comments in
  Rust source except the two markers, commit with hooks).
- Follow the lele syntax conventions in this crate's code where they don't fight the compiler API
  (one public fn per file, test_usage, domain imports). Running `lele_lint` on this crate is **not** required
  (the user lints only `lele_lint` and `freenet_libp2p_bevy_plugin`).
- The compiler's internal API changes between nightlies. **Never guess an API**: the pinned toolchain ships the
  compiler source. Grep it:
  `~/.rustup/toolchains/nightly-2026-09-01-x86_64-unknown-linux-gnu/lib/rustlib/rustc-src/rust/compiler/`.
  Verified present there: `rustc_driver_impl::Callbacks::after_analysis`, `rustc_middle::ty::Instance::try_resolve`,
  `rustc_middle::mir::ConstOperand::check_static_ptr`.
- Reference implementations to read, not copy blindly: Clippy's driver (`rust-lang/rust-clippy`, `src/driver.rs`:
  `RUSTC_WORKSPACE_WRAPPER` handling, passing through non-workspace crates, tracking a config file in dep-info);
  the rustc-dev-guide (https://rustc-dev-guide.rust-lang.org/) chapters on `rustc_driver`/`rustc_interface` and on MIR.

## 4. Current state (verified)

- `lele_function_taxonomy/` never uses the compiler: no `extern crate rustc_*` anywhere. The `rustc-private`
  feature only gates a `syn` walk (`src/run.rs`, which logs "TyCtxt wiring deferred").
- Classification is substring matching on token text (`src/run.rs` `FnVisitor`, `tests/factory.rs`):
  `'static` anywhere ⇒ "dishonest"; aliases escape; only free `fn`s in `src/`; calls keyed by bare name;
  method calls ignored; one-pass, order-dependent propagation; diagnostics at line 1; code `TAX001`.
- `rust-toolchain.toml` pins `nightly-2026-09-01` with `rustc-dev`, `llvm-tools-preview`, `rust-src`;
  that toolchain **is installed via rustup** on this machine (`/usr/bin/rustup`, `rustc-dev` and `rustc-src` present).
- **But** inside `devenv shell` the `rustc`/`cargo` on `PATH` come from Nix: `rust-nightly 2026-09-12`, **without**
  `rustc-dev`. `rust-toolchain.toml` is ignored there. So the driver cannot currently be built by the devenv tasks.
- `lele:taxonomy_check` (in ~20 crates' `devenv.nix` + the template) runs
  `cargo run --manifest-path ../lele_function_taxonomy/Cargo.toml --features rustc-private -- --manifest-path ./Cargo.toml`.
  It runs as a pre-commit hook in many crates.
- `lele.toml` `[honesty]` sections exist in several crates (`declared_honest`, `declared_dishonest`,
  `honesty_depth`, `entry_allowlist`).
- lele_lint's `lele.toml` schema is **strict** (commit `de21c13`): today it rejects `[[lele.boundary]]` entirely.

## 5. Design

### 5.1 Two binaries, clippy-style

- `lele-function-taxonomy` (runner, CLI unchanged: `--manifest-path`):
  1. Read `lele.toml` next to the manifest. If there is **no** `[[lele.boundary]]` with `require = "honest"`,
     print `no honest boundaries configured — nothing to check` and exit 0 **without compiling anything**
     (most crates run this hook; it must stay instant for them).
  2. Validate config: every boundary folder exists (else TAX002 error naming `lele.toml`).
  3. Run `cargo check --manifest-path <M> --lib --bins` using the **pinned toolchain's** `cargo`/`rustc`
     (see §6 T0 for how they are located), with:
     `RUSTC_WORKSPACE_WRAPPER=<path of lele-taxonomy-driver>`, a dedicated target dir
     `$HOME/.cache/cargo-target/lele_taxonomy` (persistent, no spaces in the path), `LD_LIBRARY_PATH` including the
     pinned sysroot's `lib/` (the driver links `librustc_driver-*.so`), and an env var carrying the boundary config
     path.
  4. Exit with cargo's status.
- `lele-taxonomy-driver` (`#![feature(rustc_private)]`): invoked by cargo as `driver <rustc> <args…>`.
  - If the crate being compiled is not the primary package (`CARGO_PRIMARY_PACKAGE` unset), behave as plain rustc
    (run the compiler with default callbacks).
  - Otherwise run the compiler with a `Callbacks` impl whose `after_analysis` performs §5.2–§5.4 and emits errors
    through the session's diagnostic context (`tcx.dcx()`), with real spans. Errors make cargo fail and are never
    cached as success.
  - Register `lele.toml` as a dependency of the compilation (as Clippy does for `clippy.toml` via the session's
    file dep-info), so editing the boundaries re-runs the analysis.

### 5.2 Building the call graph (MIR)

For every local body owner (`tcx.hir_body_owners()` or the pinned equivalent): functions, methods, trait default
methods, closures, coroutines (`async` bodies), consts/statics initialisers can be skipped.

- Read the body's MIR (`tcx.optimized_mir(def_id)`; if unavailable in `check` mode, use the earliest MIR that has
  calls resolved — T0 decides; `instance_mir`/`mir_drops_elaborated_and_const_checked` are candidates).
- **Call edges:** every `TerminatorKind::Call` whose callee operand has type `FnDef(def_id, args)`:
  - if `def_id` is a trait item, try `Instance::try_resolve(tcx, typing_env, def_id, args)`; on success use the
    resolved instance's `def_id`;
  - if it cannot be resolved because `args` contain generic parameters of the caller, the call goes through a
    parameter → **honest edge, ignore it** (the dependency is in the signature). Same for calls on `dyn Trait`
    values (virtual calls).
  - `FnPtr` calls (calling a function pointer value): ignore (it was passed in).
- **Closures and coroutines:** add an edge *parent → closure/coroutine* for every closure/coroutine defined in a
  body (its dishonesty belongs to the function that wrote it, whoever calls it later).
- **Static access:** visit all MIR constants/operands; `ConstOperand::check_static_ptr(tcx)` gives the static's
  `DefId`; also `Rvalue::ThreadLocalRef`. Mark the body as directly dishonest when the static is `static mut` or its
  type is not `Freeze` (hidden read / hidden write accordingly). `thread_local!` also shows up as calls to
  `std::thread::LocalKey::with` / `try_with` → in the I/O-root list.
- Key every node by `DefId`; print with `tcx.def_path_str(def_id)`.

### 5.3 Leaves: the I/O-root list

A callee from **another crate** is dishonest iff its `def_path_str` matches an I/O root; otherwise it is honest
(decision §2). Defaults (built in; T0 must record the *actual* strings rustc prints for each and adjust):

- clock: `std::time::SystemTime::now`, `std::time::Instant::now`
- filesystem: `std::fs::`, `std::path::Path::{exists,is_file,is_dir,metadata,read_dir,canonicalize}`
- network: `std::net::`
- environment / process: `std::env::`, `std::process::`
- console: `std::io::stdin`, `std::io::stdout`, `std::io::stderr`, `std::io::_print`, `std::io::_eprint`
  (`println!`/`eprintln!` expand to these)
- threads / globals: `std::thread::sleep`, `std::thread::LocalKey::with`, `std::thread::LocalKey::try_with`
- randomness: `rand::`, `getrandom::`
- async I/O: `tokio::fs::`, `tokio::net::`, `tokio::time::`, `tokio::io::`, `tokio::process::`, `tokio::signal::`

Matching rule: exact path, or prefix ending in `::`. Trait-impl items print like `<std::fs::File as std::io::Read>::read`;
T0 decides how to match those (likely: also match the impl's self type path, i.e. `std::fs::File` ⇒ `std::fs::`).

Config (existing `[honesty]` table, made strict; `honesty_depth`/`entry_allowlist` removed):

```toml
[honesty]
declared_dishonest = ["my_crate_dep::read_sensor"]   # extra I/O roots (paths/prefixes)
declared_honest    = ["std::time::Instant::now"]     # override: treat as honest (wins over everything)
```

`declared_honest` / `declared_dishonest` may also name **local** functions (`discovery::now_epoch`); a local
declared function is a leaf (its body is not inspected).

### 5.4 Classification and reporting

- Fixed point over the graph (reverse topological order, or a worklist until nothing changes — must be independent of
  file/iteration order). A node is dishonest if it is directly dishonest (§5.2 statics, §5.3 roots) or any callee is.
  Keep, for each dishonest node, **one** shortest witness chain to a leaf (BFS), for the message.
- Recursion / cycles must terminate (worklist with a visited set).
- Report: for every local function (not closures — they are reported through their parent) whose definition span's
  file is inside a `require = "honest"` boundary folder, and that is dishonest → one `TAX001` error at the
  function's name span, message as in §1, including the reason kind (`hidden read: clock`, `hidden write: static COUNTER`, …).
- `methods/` files: a delegate shell in a boundary file calls `methods::<type>::<method>`; dishonesty propagates to
  the shell through the call edge, so only functions physically inside the folders need checking. Do not special-case
  `methods/`.
- Codes: `TAX001` dishonest function in an honest boundary; `TAX002` invalid configuration (missing folder, unknown key).

### 5.5 Shared config with lele_lint

`[[lele.boundary]]` is read by both tools:

```toml
[[lele.boundary]]
name       = "session is pure protocol logic"
why        = "Session is driven by run.rs; it receives `now` and messages as arguments and returns actions."
folders    = ["src/discovery/session"]
cannot_use = ["tokio", "libp2p", "bevy", "std::net", "std::fs", "std::time"]   # lele_lint E036 (later)
require    = "honest"                                                          # this tool
```

- The taxonomy reads only `name`, `why`, `folders`, `require` and ignores other keys of the entry
  (lele_lint owns the strict validation).
- **lele_lint must accept this table before any crate uses it** (its schema is strict). In T4, add to lele_lint
  *by hand* (do **not** apply `docs/lele-rewrite-p3-schema.patch`):
  - `LeleSection.boundary: Vec<BoundaryEntry>` (`#[serde(default)]`);
  - `BoundaryEntry { name, why, folders, #[serde(default)] cannot_use: Vec<String>, #[serde(default)] require: Option<Requirement> }`, `deny_unknown_fields`;
  - `enum Requirement { Honest }` with `#[serde(rename_all = "lowercase")]`;
  - wire `Config(pub Option<LeleSection>)` the same way the patch does (see the patch for the exact edits to
    `load.rs`, `dunder.rs`, `apply_layout.rs`, `project.rs`), but **without** `vocabulary` (that belongs to lele P3).
  - Then update `docs/lele-rewrite-plan.md` §3/§4: P3 must not apply the patch's boundary/config parts again
    (only the vocabulary part), and E036's validation becomes "a boundary needs a non-empty `cannot_use` **or** a
    `require`".

## 6. Phases

Each phase ends with: `lele:fmt`, `lele:clippy`, `lele:nextest` passing in `lele_function_taxonomy/`, and a commit.

### T0 — spike (STOP and report to the user afterwards)

Throwaway branch-local code is fine; keep only what survives into T2.

1. **Toolchain plumbing.** Make the taxonomy crate buildable with the pinned nightly + `rustc-dev` from its own
   devenv. Evaluate, in this order, and pick the first that works:
   a. devenv's Rust toolchain-file support (check the installed devenv's `languages.rust` options for a
      `toolchainFile`-style option; the crate's `devenv.yaml` already has `fenix` and `rust-overlay` inputs, both of
      which can build a toolchain from `rust-toolchain.toml`);
   b. the rustup toolchain already installed (`rustup run nightly-2026-09-01 cargo …`).
   Record which one, why, and the exact sysroot path discovery the runner will use (e.g. capture
   `rustc --print sysroot` at build time in `build.rs` and embed it).
2. **Driver hello-world.** `lele-taxonomy-driver` run via `RUSTC_WORKSPACE_WRAPPER` on a 3-function toy crate,
   printing every call edge as `caller -> callee` with `def_path_str`, including: an aliased `SystemTime::now`,
   a method call `Instant::now().elapsed()`, a closure, an `async fn`, a trait method on a concrete type,
   a generic `C: Clock` call, `println!`, a `static` `AtomicU32`.
3. **Measure on the real target:** run the spike on `freenet_libp2p_bevy_plugin` (first run and a second unchanged
   run): wall time, peak RSS (the user has had OOM problems with Bevy; keep `CARGO_BUILD_JOBS=6`), and whether the
   second run re-analyses. Also: does editing `lele.toml` trigger re-analysis?
4. **Record the printed paths** of every §5.3 root as rustc actually prints them.

Report: chosen toolchain approach, timings/RAM, the edge dump, any API differences from §5.2, and a go/no-go.
Wait for the user before T1.

### T1 — fixtures and expected output first

Under `lele_function_taxonomy/tests/fixtures/<case>/`: tiny standalone crates (each with its own `[workspace]` table
so they are not workspace members, no registry dependencies — path dependencies only), each with a `lele.toml`
declaring an honest boundary over `src/core`, and an `expected.txt` holding the exact expected runner output
(diagnostics sorted). One integration test runs the runner on every fixture and compares; `LELE_BLESS=1` rewrites
`expected.txt` (only bless when the change is intended).

Cases (one fixture may hold several functions):
- **honest:** arithmetic; mutation through `&mut` param; call to another honest local fn; generic `C: Clock` param;
  `&dyn Clock` param; closure passed in and called; `&self` + `Cell` field mutation (decision A); read of an immutable
  `Freeze` static; call into a path-dependency crate whose function reads the clock (documents the accepted
  false negative of "unknown external = honest"); `tracing`-like logging via a local path-dep stub.
- **dishonest:** `SystemTime::now` direct; via `use … as Clock` alias; via `use std::time::*`; two-hop chain across
  modules through a `pub use` re-export; `Instant::now().elapsed()`; trait impl on a concrete type that reads
  `std::env::var`, called on that concrete type; `println!`; `static mut` write; `static COUNTER: AtomicU32`
  `fetch_add`; `OnceLock` global; `thread_local!`; closure *defined* in the function that reads the clock; `async fn`
  that reads the clock; recursion (`a → b → a → clock`); a function in `methods/<type>/` reached through an
  `#[atomic_delegates]` shell in the boundary (path-dep on `../../../../atomic_delegate_macros`);
  `declared_dishonest` on a local function; `declared_honest` overriding `Instant::now`.
- **scope:** a dishonest function **outside** the boundary → no diagnostic; a crate with no boundary → instant exit,
  no compile; a boundary folder that does not exist → `TAX002`; `lele.toml` with `honesty_depth` → `TAX002`.
- **determinism:** the same fixture with files renamed/reordered yields identical output.

Do not commit T1 alone (the tests fail until T3); commit at the end of T3.

### T2 — driver and call graph

Implement §5.1 (runner + driver) and §5.2 for real. Unit-test graph construction where possible
(pure functions over a small graph type that does not depend on `TyCtxt`; keep the `TyCtxt` code thin).

### T3 — classification, roots, reporting

Implement §5.3 and §5.4. All T1 fixtures pass. Commit T1–T3 together.

### T4 — configuration and integration

1. `[honesty]` strict: remove `honesty_depth` and `entry_allowlist` from the code; unknown keys → `TAX002`.
   Delete those two keys from every crate's `lele.toml` and from the template
   `~/.config/opencode/skills/lele-rs/references/lele-rust-config/lele.toml` (same precedent as the
   `[lele.lint.checkers]` cleanup the user approved).
2. lele_lint schema for `[[lele.boundary]]` (§5.5), with tests in `methods/config/load.rs`, golden/rule tests still
   passing, `lele_lint/RULES.md` unchanged (no new rule yet). Update `docs/lele-rewrite-plan.md` as §5.5 says.
3. `lele:taxonomy_check` exec: change it in every `devenv.nix` that has it and in the template so the runner is
   built with the pinned toolchain (per T0's choice). It must still exit instantly in crates without honest
   boundaries.
4. Adopt in the plugin: add the session boundary (§5.5) to `freenet_libp2p_bevy_plugin/lele.toml` and run the task.
   If it reports dishonest functions (expected: `now_epoch`-style clock reads), **do not weaken the boundary and do
   not refactor the plugin**: leave the boundary out of the commit and report the findings to the user
   (the sans-IO refactor is separate work).

### T5 — cleanup and docs

- Delete the syn heuristics and their tests: `analyze_stub.rs`, `check_depth.rs`, the `syn` walk in `run.rs`,
  `tests/factory.rs`, `tests/{pure,honest,dishonest}_functions/` (their cases now live in T1 fixtures), the
  `syn`/`quote`/`proc-macro2` dependencies if unused, the `rustc-private` feature (the driver is no longer optional).
  Rename `HirHonestyResult` etc. to names that match reality.
- Update `Cargo.toml` `description`, the `definition-function-taxonomy` skill only if a definition changed (it should
  not), the `lele-rs` skill index (one line: what `taxonomy_check` does now and that it needs honest boundaries),
  and `docs/lele-rewrite-plan.md` §5.2 item 3 (RATIONALE: boundaries now have two checks — imports by lele_lint,
  honesty by the taxonomy).

## 7. Out of scope

- Checking functions outside boundaries; any crate-wide depth rule.
- Analysing external crates' bodies.
- Interior mutability through `Arc`/`Rc`-shared fields (decision A treats it as honest; revisit only if the user asks).
- Fixing the other crates (they are known to be behind).
