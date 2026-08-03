# lele_lint

A standalone CLI tool that enforces the lele-syntax-rs skill conventions
by walking the source tree and checking `.rs` files.  Emits diagnostics
in clippy-compatible format for IDE/CI integration.

## Motivation

The lele-syntax-rs skill defines ~20 rules covering file structure,
module organization, import style, and code patterns.  A mechanical
linter catches violations early, reduces review burden, and works in CI.

Existing tools (clippy, dylint, marker) operate on the compiler's
internal AST/HIR — they cannot check filesystem-level rules (filenames,
mod.rs purity, directory layout).  lele_lint fills that gap with a
walkdir-based approach, and also performs syn-based AST checks for
syntax-level rules.

## Design Principles

- **Modular checkers:** Each rule is an independent checker implementing
  a `Checker` trait.  Toggling a checker on/off is trivial.
- **No nightly:** Uses stable Rust only (walkdir + syn).
- **Clippy-compatible output:** `file:line:col: error[CODE]: message`
  format for rust-analyzer and GitHub Actions.
- **Configurable:** A `lele_lint.toml` at the project root enables
  per-project rule toggles, bevy mode, and path exclusions.
- **Fast by default:** Walks `src/` once, parses files once, runs all
  checkers.  Optional `--diff` mode for changed files only.

## V1 Rule Coverage

### Filesystem / Structural (walkdir)

**1. One primary item per file, filename matches item name**
    - Each `.rs` file contains exactly one `pub struct`, `pub enum`, or
      `pub fn` (the primary item).
    - The filename is the snake_case equivalent of that item's name.
      Example: `pub struct GameConfig` lives in `game_config.rs`.
    - Exempt: `mod.rs`, `lib.rs`, `constants.rs`, files in `tests/`.
    - Opt-out: add `// lele_lint: allow E001` to skip this check.

**1a. Helper function limit**
    - Maximum 2 non-`pub` helper functions at the top level (outside
      `impl` blocks and `#[cfg(test)]` modules).
    - Excess helpers should be extracted into `<type>_<function>.rs`
      method files as thin delegates.  Prefer extracting pure/stateless
      functions; keep only context-specific ones inline.
    - Opt-out: add `// needed helper:` anywhere in the file to justify
      keeping more than 2 helpers.

**2. All filenames and directories are snake_case**
   - Every `.rs` filename and every directory under `src/` uses
     `snake_case`.  No exceptions.

**3. Method files are private**
   - Files matching the pattern `<struct>_<method>.rs` in a domain
     folder are identified as method files.
   - They must be declared with `mod` (private) in their parent
     `mod.rs` — never `pub mod`.
   - They must never appear in a `pub use` re-export.

**4. No cross-domain re-exports in mod.rs**
   - A `mod.rs` may only `pub use` items declared in files or
     subdirectories within its own directory.
   - `pub use crate::other_domain::Type;` in a `mod.rs` is a violation.
   - Cross-domain re-exports belong in `lib.rs` only.

**5. bevy_systems/ not re-exported at domain root**
   - When bevy mode is enabled: domains may contain a `bevy_systems/`
     subfolder for Bevy system functions.
   - The domain's `mod.rs` declares `pub mod bevy_systems;` but does
     NOT `pub use` individual systems at the domain root.
   - `bevy_systems/mod.rs` must flatten via `pub use` so the consumer
     path is `domain::bevy_systems::system_name` (not
     `domain::bevy_systems::system_name::system_name`).

**6. test_usage present in non-exempt files**
   - Every `.rs` file whose primary item is a non-trivial function
     (branching, arithmetic, I/O, allocation) must contain a
     `#[cfg(test)] mod tests { ... }` block with a `test_usage` test.
   - Exempt: type-only definitions (pure struct/enum with zero impl
     blocks beyond `Default`), `constants.rs`, struct files where
     `impl Default` is the only non-delegate impl block.

**7. Tests in same file as primary item**
   - Unit tests live in a `#[cfg(test)]` module at the bottom of the
     same file.  No separate `tests/` directories for unit tests.
   - Integration tests in `tests/` or `integration_tests/` are exempt.

**8. Bevy systems in bevy_systems/ subfolder only (bevy mode)**
   - When bevy mode is enabled: functions registered with
     `app.add_systems()` must live in `<domain>/bevy_systems/`.
   - A system function is identified by its signature containing Bevy
     system parameters (`Res`, `ResMut`, `Query`, `Commands`,
     `MessageWriter`, `MessageReader`).

### AST / Syntax (syn)

**9. No positional struct field access (`.0`, `.1`)**
   - Structs defined in the crate must use named fields only.
     Accessing them by position (`.0`, `.1`) is a violation.
   - Exempt: external crate types (`text.0` on types not defined in
     this crate) and anonymous tuples.

**10. No trivial accessor methods**
    - A method is a trivial accessor when:
      1. It reads or writes exactly one `pub` field of `self`.
      2. The body is a single expression/assignment (no computation,
         validation, or side effect).
      3. It is not required by a trait implementation.
    - Violation: remove the method; callers access the `pub` field directly.

**11. Domain-prefix imports**
    - All domain types and functions are accessed through their domain
      module prefix: `use crate::clicker;` then `clicker::Config`,
      `clicker::increment()`.
    - Violation: `use crate::clicker::Config;` (direct type import).
    - Exempt: `super::` imports in thin delegate struct files (see #12)
      and `use super::function;` in `#[cfg(test)]` modules.
    - Also exempt: `pub use` re-exports in `mod.rs` and `lib.rs`.

**12. Thin delegate format**
    Thin delegate `impl` blocks must follow all of:
    - **a.** The `impl` block is annotated with `#[rustfmt::skip]`.
    - **b.** The file imports method modules via `use super::<module>;`.
    - **c.** Each method body is a 2-segment dispatch:
      `module_name::function_name(self, args)`.

**13. Constructor impl blocks NOT `#[rustfmt::skip]`**
    - `impl Default for Type` blocks with a real body must NOT use
      `#[rustfmt::skip]`.
    - Any non-delegate `impl` block whose methods are constructors
      (`fn() -> Self` or `fn(...) -> Self` without `self` receiver, and
      with a real body) must NOT use `#[rustfmt::skip]`.
    - Only thin-delegate `impl` blocks (where every method body is a
      single delegation call) get `#[rustfmt::skip]`.

**14. Logging uses tracing! macros (deferred to skill)**
    - This rule was removed from the linter in v1.
    - See Non-Goals below for rationale.  The rule remains in the
      lele-syntax-rs skill as agent guidance.

## Non-Goals (V1)

Each item below is intentionally excluded from v1 with a rationale.

### No `tracing!` vs `println!` enforcement

**Why:** `println!`/`eprintln!`/`dbg!` have legitimate uses (CLI output,
debugging, `main.rs` startup) that cannot be mechanically distinguished
from logging calls.  A linter cannot know whether a given `println!` is
"user-facing output" or "should be a tracing macro."  The rule remains
in the lele-syntax-rs skill where an agent can apply human judgment.

### No `.unwrap()`, `.expect()`, `panic!()`, `todo!()` checks

**Why:** Detecting these requires a `--strict` mode that is on by
default for some projects and off for others (tests, examples,
prototyping).  Additionally, distinguishing "this `.unwrap()` is
unreachable and safe" from "this `.unwrap()` hides a real error path"
requires control-flow and type analysis beyond syn's single-file scope.
Planned for v2 as an opt-in strict mode with per-function
allow/deny annotations.

### No comment enforcement

**Why:** The lele-syntax-rs skill discourages comments ("No Comments")
in favor of self-documenting code, but:
- Comments are allowed in practice (license headers, `// SAFETY:`,
  temporary `// FIXME:`).
- Distinguishing "minimum necessary" from "excessive" is subjective
  and cannot be mechanically decided.
- Enforcing a blanket ban would produce noise and false positives on
  legitimate documentation comments (`///` doc comments are always
  allowed).

The rule set is designed so that well-structured code naturally
minimizes the need for comments — the linter enforces the structure,
not the absence of text.

### No type-level analysis

**Why:** syn parses individual files and cannot resolve types across
module boundaries.  Checks that require knowing "does this type
implement `Default`?", "is this an external crate type?", or "what
trait methods does this type have?" need rustc's HIR/MIR.  Adding this
would require a nightly rustc dependency (via dylint or marker).  The
v1 AST checks are scoped to what syn can reliably determine from a
single file's syntax tree.

### No auto-fix suggestions

**Why:** Many lele-syntax-rs violations require human judgment to fix:
choosing a filename, naming a `test_usage`, deciding where to move a
cross-domain re-export.  Auto-fix (`--fix`) would produce incorrect
results for these cases.  v2 may add auto-fix for purely mechanical
rules (e.g., adding `#[rustfmt::skip]` to a thin delegate block).

### No cross-crate analysis

**Why:** lele_lint operates on the local crate's `src/` tree.  It does
not compile dependencies or workspace members.  Checking rules across
crate boundaries (e.g., "does this workspace crate follow the same
conventions?") would require Cargo workspace resolution and is out of
scope for v1.

## Usage

```bash
cargo install lele_lint

# In a project root:
lele_lint                    # check entire src/ tree
lele_lint --bevy             # enable bevy-specific checks
lele_lint --diff HEAD~1      # check changed files only
lele_lint --checker-list     # list all active checkers
lele_lint --explain E001     # show documentation for error code E001
```

## Configuration (lele_lint.toml)

```toml
# Project root
[lele_lint]
bevy_mode = false                # enable bevy-specific checkers
exclude = ["src/generated/"]     # paths to skip

[checkers]
domain_import = true             # toggle individual checkers
no_positional = true
no_trivial_accessors = true
# ...
```

## CI Integration

```yaml
- uses: actions-rs/cargo@v1
  with:
    command: install
    args: lele_lint
- run: lele_lint --error-format github
```

## Crate Structure

```
lele_lint/
  Cargo.toml
  src/
    main.rs                      # CLI: args, walk src/, orchestrate
    lib.rs                       # library root
    checker.rs                   # trait Checker, Diagnostic, Severity
    reporting.rs                 # clippy-format + github-format output
    config.rs                    # lele_lint.toml parsing
    project.rs                   # file discovery + parsing + module tree
    module_info.rs               # mod.rs declaration/reexport parsing
    error.rs                     # thiserror Error enum
    checkers/
      mod.rs                     # build_checkers() registry
      atomic_file.rs             # rule 1 (E001)
      snake_case_files.rs        # rule 2 (E002)
      method_visibility.rs       # rule 3 (E003)
      no_cross_domain_reexport.rs # rule 4 (E004)
      test_usage.rs              # rule 6 (E006)
      test_inline.rs             # rule 7 (E007)
      no_positional.rs           # rule 9 (E009)
      no_trivial_accessors.rs    # rule 10 (E010)
      domain_import.rs           # rule 11 (E011)
      thin_delegates.rs          # rule 12 (E012)
      constructor_no_skip.rs     # rule 13 (E013)
      helper_count.rs            # rule 1a (E015)
      *_register.rs              # method files (PRIVATE, one per checker)
```
