---
name: lele-syntax-rs
description: Use for Rust code in this project. Enforces atomic file structure (snake_case files everywhere, co-located domain folders), module flattening, thiserror error handling, inline testing, domain-prefix imports, no trivial accessors, and struct field shape (single-field structs are tuple newtypes with #[derive(Deref)]; multi-field structs use named fields).
---

# SYNTAX & ARCHITECTURE GUIDELINES

## Template Convention

All examples use template variables to remain project-agnostic:

| Variable | Meaning | Example replacement |
|---|---|---|
| `{{module}}` | Domain module path | `clicker`, `player`, `combat` |
| `{{Type}}` | PascalCase type name | `Config`, `Credentials` |
| `{{type}}` | snake_case type name (lowercase of `{{Type}}`) | `config`, `credentials` |
| `{{function}}` | snake_case function name | `authenticate`, `broadcast` |
| `{{subfolder}}` | User-chosen subfolder within a domain | `plugin`, `gui` |
| `{{crate}}` | Crate name (snake_case) | `my_crate`, `bevy_p2p` |

Replace these with actual names from your project. Never use template variables literally in code — the compiler will reject them.

## 1. Rule Priority
This file's rules override standard Rust conventions. Treat this file as the absolute source of truth for architecture, naming, file structure, and error handling.

## 2. Domain / Feature Mapping
The project is divided into isolated domain/feature modules. Each domain lives in a single folder under `src/` (e.g., `src/clicker/`, `src/player/`). Structs, their methods, system functions, and supporting types are all co-located in the domain folder. There is no `structs/`/`methods/`/`system/` split — everything for a domain lives together. In these rules, we use `{{module}}` as a template variable meaning "your domain folder path" (e.g., `clicker`, `player`). **IMPORTANT: `{{module}}` is not valid Rust syntax. Never use it literally — you must replace it with the actual module name.**

Cross-cutting types that span domains may live in a dedicated module (e.g., `common/`). Do not invent root modules.

## 3. Atomic File Structure & Naming (CRITICAL)

Every file must contain exactly **one** primary logic unit (one function, one struct, or one enum).
**Rule:** The filename MUST have the exact same name as the core item inside it.

### Domain Layout (Co-located)

All code for a domain module lives in a single flat folder:

```
src/
  lib.rs                         # pub mod {{module}}; + crate-level re-exports
  {{module}}/                    # domain folder — structs, methods, systems, types all co-located
    mod.rs
    {{type}}.rs                  # struct definition + Default + thin delegates
    {{type}}_{{function}}.rs     # method free function + test_usage  (PRIVATE module)
    {{name}}.rs                  # pure enum / error type / message struct
    {{function}}.rs              # system function or domain-level free function  (PUBLIC)
    constants.rs                 # grouped module-level constants  (optional)
```

**All filenames are snake_case.** Every file and directory name in the crate MUST use snake_case. Type names (struct, enum, trait) are still PascalCase in Rust source code — the filename is the snake_case equivalent. No `#[path]` attributes, no `non_snake_case = "allow"` — since all filenames are snake_case, `pub mod config;` naturally resolves to `config.rs` with no collision.

### Struct File (`{{module}}/{{type}}.rs`)

Contains struct definition, `impl Default` (real body, if any), associated constants (real bodies), plus ALL other `impl` blocks as **thin delegates** calling sibling method files. No method bodies, no business logic, no tests.

> **Why `impl Default` is an exception:** `Default` is uniformly trivial (one-liner constructor or literal fields), exempt from testing as a trivial method (Rule 8), and `{{type}}.rs` with only `impl Default` + thin delegates is exempt from the struct-level `test_usage` requirement (Rule 8).

> **`#[rustfmt::skip]` on thin delegate impl blocks:** Annotate every thin delegate `impl` block with `#[rustfmt::skip]` to preserve one-liner format. The `impl Default` block (real body) is NOT skipped.

> **Clarification — struct def goes in `{{module}}/{{type}}.rs`, not in mod.rs:** The struct definition MUST live in its own `.rs` file. Never put it inside a `mod.rs`. This keeps mod.rs pure per Rule 6.

**Layout (in order):**
1. `struct` definition
2. `impl TypeName { pub const ... }` — associated constants, real bodies
3. `impl Default` — real body
4. All other `impl` blocks — thin delegates

**Example:**

```rust
// {{module}}/config.rs
use super::config_new;

pub struct Config {
    pub timeout_secs: u64,
}

impl Default for Config {
    fn default() -> Self { Self { timeout_secs: 30 } }
}

#[rustfmt::skip]
impl Config {
    pub fn new() -> Self { config_new::new() }
    pub fn coop() -> Self { config_new::coop() }
}
```

### Method File (`{{module}}/{{type}}_{{function}}.rs`)

Contains a single free function matching the method name exactly. The module is **PRIVATE** — declared with `mod` (not `pub mod`) in `mod.rs`. Method files are never imported directly; they are consumed exclusively through the struct's thin delegates.

The filename follows the pattern `<struct_name>_<method>.rs` so that method files of the same struct sort together alphabetically in the flat domain folder.

```rust
// {{module}}/config_new.rs
use super::config::Config;

pub fn new() -> Config { Config::default() }

#[cfg(test)]
mod tests {
    use super::new;
    use crate::{{module}}::Config;

    #[test]
    fn test_usage() {
        let config = new();
        assert!(config.timeout_secs > 0);
    }
}
```

**Thin delegate dispatch from struct file:**

```rust
// {{module}}/config.rs
use super::config_new;

#[rustfmt::skip]
impl Config {
    pub fn new() -> Self { config_new::new() }    // 2 segments
}
```

Both files are siblings in `{{module}}/`. The struct file imports the method module with `use super::config_new;` and calls it as `config_new::new()`. No deep crate paths.

### Method Files Are Private

Method modules are declared with `mod` (private) in the domain `mod.rs`. They are NOT re-exported. Only the struct type and system functions are public:

```rust
// {{module}}/mod.rs
mod config;                     // struct module — private
mod config_new;                 // method module — PRIVATE
mod config_coop;                // method module — PRIVATE
pub mod increment;              // system function — PUBLIC

pub use config::Config;         // type is public
pub use increment::increment;   // system is public
```

External consumers can only call `config.new()` through the struct's public thin delegate. `crate::{{module}}::config_new::new()` is a private module and will not compile.

### Grouping (User-Directed)

The base structure is flat. Method files and struct files are siblings in one folder. When a domain grows, the user may introduce subfolders. Method files inside subfolders keep the same `<struct>_<method>.rs` naming:

```
{{module}}/
  mod.rs
  plugin/
    mod.rs
    plugin.rs
    plugin_build.rs
  state/
    mod.rs
    state.rs
    state_new.rs
    state_increment.rs
  command.rs                     # stays flat
  event.rs                       # stays flat
  increment.rs                   # stays flat
```

No automated threshold triggers grouping — the user decides. The only rules are:
- Method filenames always use `<struct>_<method>.rs`, even inside subfolders
- Subfolder items are NOT re-exported at the domain root; consumers access them through the subfolder path
- `mod.rs` in every directory follows Rule 6 (module tree only)

```rust
// {{module}}/plugin/mod.rs
mod plugin;                     # struct — private, but pub use below
mod plugin_build;               # method — PRIVATE

pub use plugin::ClickerPlugin;  # type is public
```

Consumer path: `use crate::{{module}}::plugin::ClickerPlugin;`

> **Delegation call rule:** When a method file calls another method of the same struct, it MUST route through the struct's public API (e.g., `Config::coop()`), not call the other method file directly. The struct's thin delegates are the authoritative API surface. Example chain: `Config::lan_coop()` → thin delegate → `config_lan_coop::lan_coop()` → calls `Config::coop()` → thin delegate → `config_coop::coop()`.

### Systems Subfolder

Bevy system functions (functions registered via `app.add_systems()`) live in a `bevy_systems/` subfolder within the domain. This prevents filename collisions between Component types and system functions (e.g., `IncrementButton` Component at `increment_button.rs` vs a system that would share the same filename).

```
{{module}}/
  bevy_systems/
    mod.rs
    poll_events.rs              # system function  (PUBLIC)
    handle_input.rs             # system function  (PUBLIC)
  increment_button.rs           # Component struct
  plugin.rs                     # Plugin struct
  plugin_build.rs               # plugin build method (PRIVATE)
```

Non-system functions (core logic, method files, plugin builds) stay flat in the domain folder. `bevy_systems/mod.rs` must flatten system stutter via `pub use`:

```rust
// {{module}}/bevy_systems/mod.rs
pub mod poll_events;
pub mod handle_input;

pub use poll_events::poll_events;
pub use handle_input::handle_input;
```

Systems are NOT re-exported at the domain root — consumers access them as `{{module}}::bevy_systems::poll_events`.

### Named Defaults

A "named default" is a preset constructor (e.g., `Config::coop()`, `Config::pvp()`). It follows the same decomposition rule — goes in a method file `{{type}}_{{name}}.rs`.

A method qualifies as a named default when ALL hold:
1. Returns `{{Type}}`, takes no `self` receiver
2. Return value is statically determined (literal field values, no params)
3. Purpose is to provide a preset configuration variant

Examples: `Config::coop()`, `Config::pvp()`
Counterexamples: `Config::new()` — generic constructor; `Config::with_auto_accept(mut self, ...)` — builder, takes self

**Benefits of this decomposition:**
- `{{module}}/{{type}}.rs` shows every public method signature at a glance.
- Individual files can be `#[cfg(feature = "...")]`-gated.
- Each file carries its own self-contained test.
- The struct definition remains a minimal, readable declaration.

**Feature gating convention:** Do not add feature flags unless explicitly requested.

**Helper exception:** A few small private helper functions used **exclusively by the file's single primary item** are permitted in the same file (up to 2 unannotated). Beyond that, annotate the ones that must stay with `// needed helper:` on the line directly above each individual function (skipping blank lines and `#[...]` attributes) — the annotation only excuses that one function, not the whole file. A file may have at most 1 top-level function that is `pub` or `pub(crate)` — its single core function. A second `pub`/`pub(crate)` function in the same file has no opt-out and must move to its own file, even for trivial one-liners.

### Constants

#### Associated Constants (belonging to a struct type)

A constant meaningful only in the context of a **single** struct type MUST be an associated constant inside `{{module}}/{{type}}.rs`:

```rust
// {{module}}/{{type}}.rs
pub struct {{Type}} { pub inner: libp2p::gossipsub::IdentTopic }

impl {{Type}} {
    pub const GAME_TOPIC_STR: &str = "{{crate}}_p2p_game";
}
```

**Criterion — associated vs. module-level:** An associated constant if **all** hold:
1. Its value is only meaningful for one specific struct type.
2. It is only referenced by that type's own methods.
3. No other type, function, or module reads it.

If any code outside the struct's own files references it, it MUST be a module-level constant in `constants.rs`.

#### Module-level Constants

A constant spanning multiple types or referenced by module-level functions goes in a grouped `constants.rs` file:

```
{{module}}/
  mod.rs                  # pub mod constants; pub use constants::*;
  constants.rs            # grouped pub const definitions
```

```rust
// {{module}}/constants.rs
pub const HASTE: Ability = Ability::Static { ... };
pub const VIGILANCE: Ability = Ability::Static { ... };
```

**No `test_usage` required** — pure value declarations are exempt from testing (Rule 8).

## 4. Type Naming — No Restrictions

Type names follow standard Rust conventions. There are no echo-name prohibitions — the unified domain-prefix import style (Rule 5B) provides full disambiguation through the module path, so naming constraints are unnecessary.

## 5. Module Exporting & Flattening (CRITICAL)

### A. Exporting (Inside `mod.rs`)

Flatten single-function and single-struct files in their parent `mod.rs` using `pub use` to prevent stutter. Method files are NEVER re-exported — they are private modules.

**Struct types** (public):

```rust
// {{module}}/mod.rs
pub mod {{type}};
pub use {{type}}::{{Type}};     // Flatten: {{module}}::{{Type}} not {{module}}::{{type}}::{{Type}}
```

**Method files** (PRIVATE):

```rust
// {{module}}/mod.rs
mod {{type}}_{{function}};      // private — no pub, no pub use
```

**`bevy_systems/` subfolder:**

```rust
// {{module}}/mod.rs
pub mod bevy_systems;           // declared — items inside NOT re-exported at domain root
```

Systems in `bevy_systems/` are NOT re-exported at the domain root. `bevy_systems/mod.rs` must flatten via `pub use` so the consumer path is `{{module}}::bevy_systems::handle_cli` (not `{{module}}::bevy_systems::handle_cli::handle_cli`). See Rule 3 for the `bevy_systems/mod.rs` pattern.

**User-chosen subfolders:**

```rust
// {{module}}/mod.rs
pub mod {{subfolder}};          // declared — items inside NOT re-exported
```

Items inside subfolders are accessed through their folder path (e.g., `{{module}}::plugin::ClickerPlugin`). Do NOT re-export them at the domain root.

**`constants.rs`:**

```rust
// {{module}}/mod.rs
pub mod constants;
pub use constants::*;           // constants live in the value namespace — safe glob
```

#### mod.rs Boundary (CRITICAL)

A `mod.rs` may ONLY `pub use` items declared in files or subdirectories within its own directory. Cross-domain re-exports are FORBIDDEN in `mod.rs`.

```rust
// ✓ Allowed — items from files declared in this mod.rs
pub mod config;
pub use config::Config;

// ✓ Allowed — items from subdirectory declared in this mod.rs
pub mod bevy_systems;
// (no pub use at domain root — accessed via {{module}}::bevy_systems::handle_cli)

// ✗ Forbidden — reaching across domains
pub use crate::freenet::Client;   // belongs in lib.rs, not clicker/mod.rs
```

Cross-domain re-exports that need to be available at the crate root go in `lib.rs` ONLY.
```

### B. Importing (Inside Consumer Files)

ALL domain items (types, functions, systems) are accessed through the domain module prefix. Only external crate types use direct import.

| What you're importing | Style | Example |
|---|---|---|
| **Everything from a domain** | Import parent domain, call through it | `use crate::{{module}};` → `{{module}}::{{Type}}`, `{{module}}::{{function}}()`, `{{module}}::bevy_systems::{{function}}()` |
| **External crate types** | Import directly | `use extern_crate::Type;` |

```rust
// ✓ Correct — ALL domain items via domain prefix
use crate::clicker;

clicker::Plugin::default();
clicker::increment(&mut state, 1);
clicker::bevy_systems::handle_cli();

// ✓ Correct — external crate types
use bevy::prelude::*;

// ✗ Wrong — direct import of domain type
use crate::clicker::Config;

// ✗ Wrong — direct import of domain function
use crate::clicker::increment;

// ✗ Wrong — importing a PRIVATE method module
use crate::clicker::config_new;
```

Methods are never imported directly — they are called through the struct's thin delegates.

## 6. `mod.rs` — Module Tree Only (No Logic, No Exceptions)

A `mod.rs` file builds the module tree and flattens exports. It must NOT contain any business logic, struct definitions, or data.

**Rule:** A `mod.rs` may contain ONLY:
- `pub mod` declarations
- `mod` declarations (for private method files)
- `pub use` re-exports

Everything else is **strictly forbidden**:
- ❌ Struct/enum definitions
- ❌ `impl` blocks (methods, trait impls)
- ❌ Functions (including private helpers)
- ❌ Constants or statics
- ❌ `#[cfg(test)]` modules
- ❌ Trait definitions

✅ **Allowed — pure re-export:**

```rust
// {{module}}/mod.rs
pub mod config;
pub use config::Config;

mod config_new;            // method file — PRIVATE
mod config_coop;           // method file — PRIVATE
```

## 7. Error Handling (Strict Constraints)
- **Never use `.unwrap()`, `.expect()`, or `panic!()`.** All errors must be gracefully propagated.
- **Always use `thiserror`.** Define strongly typed, domain-specific error enums.

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid credentials provided")]
    InvalidCredentials,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Configuration error: {0}")]
    Config(String),
}
```

## 8. Testing Rules (Inline)

Tests must live in the exact same file as the core item. Do not create separate `tests/` directories or `test.rs` files. Append a `#[cfg(test)]` module at the bottom of the file.

Every file whose primary item is a non-trivial function (branching, arithmetic, I/O, or allocation) MUST contain a `test_usage` test that exercises the primary item in a way that mirrors real consumption.

**Exemption — type-only definitions:** Pure struct/enum with zero `impl` blocks → no `test_usage` required.

**Struct files with hand-written impl blocks — test_usage required:** A `{{module}}/{{type}}.rs` with any hand-written `impl` block (beyond `impl Default` alone) MUST contain a `test_usage` test that:
1. Constructs the struct.
2. Exercises it through the primary integration path.
3. Asserts on an observable outcome.

**Exemption — thin-delegate struct files:** If `impl Default` is the only non-thin-delegate `impl` block, no `test_usage` required.

**Exemption — trivial methods:** One-liner accessor/delegating methods with no branching, arithmetic, or I/O are exempt.

**Exemption — constant-only definitions:** `constants.rs` is exempt.

**Opt-out:** Add `// no test_usage necessary` as the last line of the file to exempt it from this requirement.

**Context-dependent items (e.g., framework systems):** Construct a minimal working context inside the test. See framework-specific skills for patterns.

```rust
// {{module}}/{{type}}_{{function}}.rs
use crate::{{module}};
use super::{{type}}::{{Type}};

pub fn {{function}}() -> {{Type}} { {{Type}}::default() }

#[cfg(test)]
mod tests {
    use super::{{function}};

    #[test]
    fn test_usage() {
        let result = {{function}}();
        assert!(result.timeout_secs > 0);
    }
}
```

**Plugin test example:**

```rust
// {{module}}/plugin.rs
use super::plugin_build;
use bevy::prelude::*;

pub struct Plugin;

#[rustfmt::skip]
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) { plugin_build::build(self, app) }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use super::Plugin;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(Plugin);
    }
}
```

**Imports in tests:** Follow Rule 11. `super::` is allowed for same-file items. Cross-domain types use `crate::`.

## 9. Universal Code Style

- **No Comments:** Do not write comments. Code must be self-documenting.
- **Clarity over cleverness.**
- **Early returns:** Use `?` or `return` to reduce nesting.
- **Indentation:** 4 spaces.
- **Thin delegates `#[rustfmt::skip]`:** Every thin delegate `impl` block (Rule 3) MUST use `#[rustfmt::skip]`. `impl Default` blocks (real bodies) are NOT skipped.
- **Logging:** Use `tracing!` macros.
  ```rust
  tracing::debug!(target: "module_name", var_name = var.value);
  ```

## 10. Standard Build & Verification Routine

Verify changes with:
```bash
cargo build --all-targets
cargo clippy -- -D warnings
cargo fmt -- --check
cargo test --all-targets
```

## 11. Import Style

Production code imports follow two rules:
1. **Same-domain type/function references:** Use `use crate::{{module}};` then `{{module}}::Type`, `{{module}}::function()`. Consistent with cross-domain style.
2. **Method-file dispatching:** Use `super::` for importing sibling private method files (thin delegate dispatch only).
3. **External crate types:** Import directly.

### Import by type

| What you're importing | Style | Example |
|---|---|---|
| **Domain type/function (any domain)** | Module prefix | `use crate::clicker;` → `clicker::Config`, `clicker::increment()` |
| **Method file (thin delegate dispatch)** | `super::` path | `use super::config_new;` → `config_new::new()` |
| **External crate types** | Import directly | `use bevy::prelude::*;` |

```rust
// ✓ Same-domain: struct file calling method — super:: for thin delegate dispatch
// In {{module}}/{{type}}.rs:
use super::{{type}}_{{function}};

// ✓ Same-domain: type ref in method file — domain prefix
// In {{module}}/{{type}}_{{function}}.rs:
use crate::{{module}};
// → {{module}}::Config in code

// ✓ Cross-domain: same style
use crate::combat;
combat::Damage::default();
```

```rust
// ✗ Wrong — direct import of domain type
use crate::clicker::Config;

// ✗ Wrong — super:: for a type reference (thin delegate only)
use super::config::Config;
```

### Imports in thin delegates

Thin delegates in the struct file use `use super::{{type}}_{{function}};` to import sibling method modules:

```rust
// {{module}}/{{type}}.rs
use super::{{type}}_{{function}};

#[rustfmt::skip]
impl {{Type}} {
    pub fn {{function}}() -> Self { {{type}}_{{function}}::{{function}}() }
}
```

The dispatch call is typically `module_name::function_name(self, ...)`, but any method body of at most 3 statements may stay inline in the type's own file instead — any shape (const access, a short struct literal, brief control flow, etc.), not just a dispatch call. Bodies longer than 3 statements must be extracted into their own method file. Either way, the impl block containing them stays inside a `#[rustfmt::skip]` block.

### Test modules — `super::` allowed for same-file access

```rust
// ✓ Correct — super:: for same-file items, domain prefix for types
#[cfg(test)]
mod tests {
    use super::{{function}};            // same-file item
    use crate::{{module}};              // domain prefix
    // → {{module}}::{{Type}} in code
}
```

### Exception — `mod.rs` re-exports only

```rust
// ✓ This is a re-export, not a consumer import
pub mod {{type}};
pub use {{type}}::{{Type}};
```

## 12. No Trivial Accessors (Getters/Setters)

A method that reads or writes a single `pub` field without any computation, validation, or side effect MUST be removed. Callers access the field directly.

### Mechanical Test

A method IS a trivial accessor when **all** hold:
1. Body is a single expression or assignment statement.
2. It reads or writes exactly one field of `self`.
3. That field is `pub`.
4. The method is not required by a trait implementation.

```
// ✗ WRONG — trivial getter, field is pub
fn tick(&self) -> u64 { self.0 }

// ✓ OK — trait impl
impl Deref for Wrapper {
    type Target = Inner;
    fn deref(&self) -> &Inner { &self.0 }
}

// ✓ OK — consuming builder (self → Self)
fn with_timeout(self, ms: u64) -> Self { Self { timeout: ms, ..self } }
```

## 13. Struct Field Shape (E018 + E009)

Field arity decides struct shape (enforced by `single_field_newtype`, E018):

- **Exactly one field** → MUST be a **tuple newtype** `pub struct X(T)` **with `#[derive(…, Deref)]`** (from `derive_more`). Access the value through deref (`*x`, method calls), never `.0`.
- **Two or more fields** → MUST use **named fields** `{ a: A, b: B }`. Tuple structs with ≥2 fields are forbidden.
- **`DerefMut`** is optional — added only when the type needs mutation through deref (`*counter += 1`). Never required.

Positional field access (`.0`, `.1`, ...) is banned everywhere (E009); `Deref` makes it unnecessary.

### Exceptions
- Types from **external crates** (not under your control).
- **Anonymous tuples** — inherently positional.

```
// ✓ CORRECT — single field → tuple newtype with Deref
#[derive(Clone, Deref)]
pub struct PlayerId(pub u64);
fn check(id: &PlayerId) -> bool { *id == 0 }

// ✓ CORRECT — two fields → named
pub struct Player { pub id: PlayerId, pub health: u32 }

// ✗ WRONG — single field with a redundant name
pub struct PlayerId { pub value: u64 }

// ✗ WRONG — single-field tuple newtype without Deref
pub struct PlayerId(pub u64);

// ✗ WRONG — two-field tuple
pub struct Pair(pub String, pub u32);

// ✓ OK — external crate
text.0 = format!("{}", count);
```

## 14. Clippy Config (CRITICAL)

Every crate must contain **both** a `[lints.clippy]` section in `Cargo.toml` and a `clippy.toml` at the crate root. These are **minimum** defaults — projects may extend them with additional lints or config, but must not weaken them. `workspace.lints.clippy` with `lints.workspace = true` in members also satisfies the `Cargo.toml` requirement.

### Cargo.toml — Minimum `[lints.clippy]`

```toml
[lints.clippy]
pedantic = { level = "deny", priority = -1 }
nursery = { level = "deny", priority = -1 }
unwrap_used = "deny"
expect_used = "deny"
indexing_slicing = "deny"
arithmetic_side_effects = "deny"
unreachable = "deny"
unimplemented = "deny"
unchecked_time_subtraction = "deny"
todo = "deny"
string_slice = "deny"
panic_in_result_fn = "deny"
panic = "deny"
exit = "deny"
as_conversions = "deny"
```

`pedantic`/`nursery` must be `{ level = "deny", priority = -1 }` (bare `"deny"` also accepted). All other entries must be `"deny"` (or `{ level = "deny" }`). Enforced as `E021`.

### clippy.toml — Minimum

```toml
allow-unwrap-in-tests = true
allow-expect-in-tests = true
allow-panic-in-tests = true
allow-indexing-slicing-in-tests = true
```

All four must be `true`. Extra keys are allowed. Enforced as `E022`.
