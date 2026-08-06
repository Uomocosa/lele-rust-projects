# STEP 2: Fix lele_lint annotation placement & helper counting

## Summary

Two bugs in lele_lint:

1. **`// no test_usage necessary`** — accepted anywhere (most files have it at L1); should only be recognized at the **end** of the file where `test_usage` would go.
2. **`// needed helper:`** — blanket file-level opt-out from E015; should only annotate **individual private functions** (comment on the line immediately before the fn). `pub(crate)` functions are NOT helpers and should not be counted.

Additionally, duplicated helper functions across `_check.rs` files will be extracted into a shared `common/` module.

---

## Part A: `// no test_usage necessary` — end-of-file placement

### A1. Linter change (`test_usage_check.rs`)

**File:** `src/checkers/test_usage_check.rs`

Change `has_test_usage_opt_out` to scan only the **last 5 lines** of the file:

```rust
fn has_test_usage_opt_out(project: &Project, rel_path: &Path) -> bool {
    let entry = match project
        .entries
        .iter()
        .find(|e| e.relative_path == rel_path && e.kind == EntryKind::File)
    {
        Some(e) => e,
        None => return false,
    };
    let content = match std::fs::read_to_string(&entry.absolute_path) {
        Ok(c) => c,
        Err(_) => return false,
    };
    let lines: Vec<&str> = content.lines().collect();
    let start = if lines.len() > 5 { lines.len() - 5 } else { 0 };
    lines[start..].iter().any(|line| line.trim().starts_with("// no test_usage necessary"))
}
```

Effect: files with the annotation at L1 will now fail E006 ("file must contain test_usage or opt-out").

### A2. Move annotations in all lele_lint source files (~55 files)

Every file under `lele_lint/src/` that has `// no test_usage necessary` at L1 must move it to the **last line** of the file. The opt-out replaces the `#[cfg(test)] mod tests { ... }` block position:

**Before (wrong):**
```rust
// no test_usage necessary

use crate::...;

pub struct Foo { ... }
```

**After (correct):**
```rust
use crate::...;

pub struct Foo { ... }

// no test_usage necessary
```

Files to fix (all under `lele_lint/src/`):

| Path | Current location | Action |
|------|-----------------|--------|
| `checker.rs` | L1 | Move to end |
| `config.rs` | L1 | Move to end |
| `config_bevy_mode.rs` | L1 | Move to end |
| `config_checker_enabled.rs` | L1 | Move to end |
| `config_load.rs` | L1 | Move to end |
| `module_info_build.rs` | L1 | Move to end |
| `print_checker_list.rs` | L1 | Move to end |
| `print_diagnostics.rs` | L1 | Move to end |
| `project.rs` | L1 | Move to end |
| `project_discover.rs` | L1 | Move to end |
| `project_find_cargo_root.rs` | L1 | Move to end |
| `project_get_parsed.rs` | L1 | Move to end |
| `checkers/mod.rs` | L1 | Move to end |
| `checkers/atomic_file.rs` | L1 | Move to end |
| `checkers/atomic_file_meta.rs` | L1 | Move to end |
| `checkers/atomic_file_register.rs` | L1 | Move to end |
| `checkers/constructor_no_skip.rs` | L1 | Move to end |
| `checkers/constructor_no_skip_check.rs` | L1 | Move to end |
| `checkers/constructor_no_skip_meta.rs` | L1 | Move to end |
| `checkers/constructor_no_skip_register.rs` | L1 | Move to end |
| `checkers/domain_import.rs` | L1 | Move to end |
| `checkers/domain_import_check.rs` | L1 | Move to end |
| `checkers/domain_import_meta.rs` | L1 | Move to end |
| `checkers/domain_import_register.rs` | L1 | Move to end |
| `checkers/helper_count.rs` | L1 | Move to end |
| `checkers/helper_count_check.rs` | L1+L3 (**duplicate**) | Move single copy to end |
| `checkers/helper_count_meta.rs` | L1 | Move to end |
| `checkers/helper_count_register.rs` | L1 | Move to end |
| `checkers/method_visibility.rs` | L1 | Move to end |
| `checkers/method_visibility_meta.rs` | L1 | Move to end |
| `checkers/method_visibility_register.rs` | L1 | Move to end |
| `checkers/no_cross_domain_reexport.rs` | L1 | Move to end |
| `checkers/no_cross_domain_reexport_meta.rs` | L1 | Move to end |
| `checkers/no_cross_domain_reexport_register.rs` | L1 | Move to end |
| `checkers/no_positional.rs` | L1 | Move to end |
| `checkers/no_positional_meta.rs` | L1 | Move to end |
| `checkers/no_positional_register.rs` | L1 | Move to end |
| `checkers/no_trivial_accessors.rs` | L1 | Move to end |
| `checkers/no_trivial_accessors_meta.rs` | L1 | Move to end |
| `checkers/no_trivial_accessors_register.rs` | L1 | Move to end |
| `checkers/single_caller_type.rs` | L1 | Move to end |
| `checkers/single_caller_type_check.rs` | L1 | Move to end |
| `checkers/single_caller_type_meta.rs` | L1 | Move to end |
| `checkers/single_caller_type_register.rs` | L1 | Move to end |
| `checkers/snake_case_files.rs` | L1 | Move to end |
| `checkers/snake_case_files_meta.rs` | L1 | Move to end |
| `checkers/snake_case_files_register.rs` | L1 | Move to end |
| `checkers/test_inline.rs` | L1 | Move to end |
| `checkers/test_inline_meta.rs` | L1 | Move to end |
| `checkers/test_inline_register.rs` | L1 | Move to end |
| `checkers/test_usage.rs` | L1 | Move to end |
| `checkers/test_usage_check.rs` | L1 | Move to end |
| `checkers/test_usage_meta.rs` | L1 | Move to end |
| `checkers/test_usage_register.rs` | L1 | Move to end |
| `checkers/thin_delegates.rs` | L1 | Move to end |
| `checkers/thin_delegates_check.rs` | L1 | Move to end |
| `checkers/thin_delegates_meta.rs` | L1 | Move to end |
| `checkers/thin_delegates_register.rs` | L1 | Move to end |

**Special case:** `checkers/no_trivial_accessors_check.rs` — has `// no test_usage necessary` on line 120 **inside** the `#[cfg(test)]` test module. This comment inside the test module is an error (opt-out goes at file end, not inside test module). Remove it; the file actually has test functions (no opt-out needed).

---

## Part B: Extract shared utilities into `common/`

Six functions are duplicated across `_check.rs` files. Extract each into its own file under `src/common/`:

### B1. New files to create

```
src/common/
  mod.rs
  to_snake_case.rs
  is_default_impl.rs
  has_rustfmt_skip.rs
  self_type_last.rs
  is_two_segment_dispatch.rs
  is_cfg_test_mod.rs
```

### B2. `src/common/mod.rs`

```rust
mod to_snake_case;
mod is_default_impl;
mod has_rustfmt_skip;
mod self_type_last;
mod is_two_segment_dispatch;
mod is_cfg_test_mod;

pub use has_rustfmt_skip::has_rustfmt_skip;
pub use is_cfg_test_mod::is_cfg_test_mod;
pub use is_default_impl::is_default_impl;
pub use is_two_segment_dispatch::is_two_segment_dispatch;
pub use self_type_last::self_type_last;
pub use to_snake_case::to_snake_case;
```

### B3. Each utility file

Pattern for each: `pub fn ... { ... }` + `#[cfg(test)] mod tests { fn test_usage() { ... } }`

| File | Function | Copied from (current duplicate locations) |
|------|----------|------------------------------------------|
| `to_snake_case.rs` | `pub fn to_snake_case(pascal: &str) -> String` | `atomic_file_check.rs:139-160`, `thin_delegates_check.rs:198-219` |
| `is_default_impl.rs` | `pub(crate) fn is_default_impl(impl_block: &ItemImpl) -> bool` | `constructor_no_skip_check.rs:72-79`, `thin_delegates_check.rs:124-132` |
| `has_rustfmt_skip.rs` | `pub(crate) fn has_rustfmt_skip(impl_block: &ItemImpl) -> bool` | `constructor_no_skip_check.rs:52-57`, `thin_delegates_check.rs:191-196` |
| `self_type_last.rs` | `pub(crate) fn self_type_last(ty: &Type) -> Option<String>` | `thin_delegates_check.rs:117-122` (named `self_type_name`), `single_caller_type_check.rs:136-140` |
| `is_two_segment_dispatch.rs` | `pub(crate) fn is_two_segment_dispatch(block: &Block) -> bool` | `thin_delegates_check.rs:175-184`, `constructor_no_skip_check.rs:96-105` (named `is_single_delegate_call`) |
| `is_cfg_test_mod.rs` | `pub(crate) fn is_cfg_test_mod(module: &ItemMod) -> bool` | `test_usage_check.rs:192-201` (named `is_cfg_test`), `single_caller_type_check.rs:218-227` |

### B4. Update `lib.rs`

Add `pub mod common;` to `src/lib.rs`.

### B5. Update callers

Replace local function definitions with `use crate::common;` and call `common::function(...)`.

| Caller file | Remove local | Replace with |
|-------------|-------------|-------------|
| `atomic_file_check.rs` | `to_snake_case` | `common::to_snake_case(...)` |
| `thin_delegates_check.rs` | `to_snake_case` | `common::to_snake_case(...)` |
| `constructor_no_skip_check.rs` | `is_default_impl`, `has_rustfmt_skip`, `is_single_delegate_call` | `common::is_default_impl(...)`, `common::has_rustfmt_skip(...)`, `common::is_two_segment_dispatch(...)` |
| `thin_delegates_check.rs` | `is_default_impl`, `has_rustfmt_skip`, `self_type_name`, `is_two_segment_dispatch` | `common::is_default_impl(...)`, `common::has_rustfmt_skip(...)`, `common::self_type_last(...)`, `common::is_two_segment_dispatch(...)` |
| `test_usage_check.rs` | `is_cfg_test` | `common::is_cfg_test_mod(...)` |
| `single_caller_type_check.rs` | `self_type_last`, `is_cfg_test_mod` | `common::self_type_last(...)`, `common::is_cfg_test_mod(...)` |

### B6. Post-extraction helper counts

After removing shared utils, private helper counts per file:

| File | Before | Extracted | Remaining |
|------|--------|-----------|-----------|
| `atomic_file_check.rs` | 4 | `to_snake_case` | 3 |
| `constructor_no_skip_check.rs` | 6 | `is_default_impl`, `has_rustfmt_skip`, `is_two_segment_dispatch` | 3 |
| `thin_delegates_check.rs` | 10 | `to_snake_case`, `is_default_impl`, `has_rustfmt_skip`, `self_type_name`, `is_two_segment_dispatch` | 5 |
| `test_usage_check.rs` | 10 | `is_cfg_test` | 9 |
| `single_caller_type_check.rs` | 10 | `self_type_last`, `is_cfg_test_mod` | 8 |
| `no_positional_check.rs` | 5 | — | 5 |
| `no_trivial_accessors_check.rs` | 4 | — | 4 |
| `domain_import_check.rs` | 5 | — | 5 |
| `method_visibility_check.rs` | 6 | — | 6 |
| `module_info_build.rs` | 3 | — | 3 |

All remaining files exceed MAX_HELPERS (2). Individual `// needed helper:` annotations needed (see Part D).

---

## Part C: Fix `// needed helper:` linter semantics

### C1. Change `count_top_level_helpers` — only truly private fns

**File:** `src/checkers/helper_count_check.rs`

**Current:**
```rust
fn count_top_level_helpers(file: &syn::File) -> usize {
    file.items
        .iter()
        .filter(|item| {
            if let syn::Item::Fn(func) = item {
                return !matches!(func.vis, syn::Visibility::Public(_));
            }
            false
        })
        .count()
}
```

**Changed:**
```rust
fn count_top_level_helpers(file: &syn::File) -> usize {
    file.items
        .iter()
        .filter(|item| {
            if let syn::Item::Fn(func) = item {
                return matches!(func.vis, syn::Visibility::Inherited);
            }
            false
        })
        .count()
}
```

Effect: `pub(crate) fn`, `pub(super) fn`, etc. are no longer counted. Only `fn foo()` (no visibility keyword) is counted. This means `project.rs`'s `walk_entries` and `parse_source_files` (both `pub(crate)`) are excluded.

### C2. Replace `has_opt_out` with annotation-based exclusion

**File:** `src/checkers/helper_count_check.rs`

`has_opt_out` (blanket file-level check) is removed. Instead, for each file, read the source text and cross-reference with the syn AST to count only un-annotated private fns.

New approach:
1. Read source text of the file
2. Build a byte-offset → line-number mapping from the source
3. For each top-level `fn` with `Visibility::Inherited`, find its starting line from the syn span
4. Check if the line immediately before it contains `// needed helper:` (after trimming)
5. Count functions without the annotation

**New function: `count_unannotated_helpers`:**
```rust
fn count_unannotated_helpers(file: &syn::File, source: &str) -> usize {
    let line_offsets = build_line_offsets(source);
    file.items
        .iter()
        .filter_map(|item| {
            let syn::Item::Fn(func) = item else { return None };
            if !matches!(func.vis, syn::Visibility::Inherited) {
                return None;
            }
            Some(func)
        })
        .filter(|func| {
            let fn_line = byte_offset_to_line(func.span().start().byte, &line_offsets);
            if fn_line == 0 {
                return true; // first line of file, can't annotate
            }
            let prev_line_idx = fn_line - 1;
            let prev_line = source.lines().nth(prev_line_idx).unwrap_or("");
            !prev_line.trim().starts_with("// needed helper:")
        })
        .count()
}
```

Helper functions for source text processing:

```rust
fn build_line_offsets(source: &str) -> Vec<usize> {
    std::iter::once(0)
        .chain(source.bytes().enumerate().filter_map(|(i, b)| {
            if b == b'\n' { Some(i + 1) } else { None }
        }))
        .collect()
}

fn byte_offset_to_line(byte_offset: usize, line_offsets: &[usize]) -> usize {
    line_offsets.partition_point(|&offset| offset <= byte_offset).saturating_sub(1)
}
```

### C3. Update `check()` to use new counting

Replace:
```rust
if has_opt_out(project, rel_path) {
    continue;
}
let helper_count = count_top_level_helpers(file);
```

With:
```rust
let source = read_file_source(project, rel_path);
let unannotated = count_unannotated_helpers(file, &source);
if unannotated > MAX_HELPERS { ... }
```

Where `read_file_source` reads the file content from `entry.absolute_path`.

### C4. Remove `has_opt_out` function entirely

The blanket file-level opt-out is no longer valid.

---

## Part D: Fix remaining files — annotation cleanup & relocation

### D1. Files with no private fns — REMOVE `// needed helper:` entirely

| File | Reason |
|------|--------|
| `project.rs` | `walk_entries` + `parse_source_files` are `pub(crate)`, not private. After C1, they're not counted. |
| `module_info.rs` | `walk_tree` is inside `#[cfg(test)]` block, not top-level. Zero private fns. |

### D2. Files with ≤2 private fns — REMOVE `// needed helper:` (within limit)

| File | Private fn count | Remaining after Part B |
|------|-----------------|----------------------|
| `helper_count_check.rs` | 2 | 2 |
| `snake_case_files_check.rs` | 2 | 2 |

These are within MAX_HELPERS=2. No annotation needed.

### D3. Files with 3+ private fns — annotate each private fn individually

For each file below, remove the existing `// needed helper: parsing utilities` comment from its current location (top-of-file or between-imports-and-check) and add individual `// needed helper:` annotations on the line immediately above each private function.

**`atomic_file_check.rs`** (3 helpers after extraction):
```rust
// needed helper: path exemption logic
fn is_exempt_path(rel_path: &Path, file_name: &str) -> bool {
// needed helper: pub item collection
fn collect_pub_items(file: &syn::File) -> Vec<PubItem> {
// needed helper: filename validation against snake_case
fn check_filename_match(name: &str, file_stem: &str, rel_path: &Path, project: &Project, diags: &mut Vec<Diagnostic>) {
```

**`constructor_no_skip_check.rs`** (3 helpers after extraction):
```rust
// needed helper: type name string extraction
fn type_name_string(ty: &syn::Type) -> String {
// needed helper: thin delegate body detection
fn is_thin_delegate(impl_block: &syn::ItemImpl) -> bool {
// needed helper: real constructor detection (non-delegate static method)
fn has_any_real_constructor(impl_block: &syn::ItemImpl) -> bool {
```

**`domain_import_check.rs`** (5 helpers):
```rust
// needed helper: struct-file detection for import exemption
fn is_struct_delegate_file(file: &syn::File) -> bool {
// needed helper: import style validation
fn check_import(item_use: &syn::ItemUse, is_struct_file: bool) -> Option<String> {
// needed helper: visibility check
fn is_pub_use(item_use: &syn::ItemUse) -> bool {
// needed helper: use tree segment collection
fn collect_use_segments(tree: &syn::UseTree) -> Vec<String> {
// needed helper: source line lookup for use statements
fn find_use_line(entries: &[crate::entry::Entry], rel_path: &Path, _item_use: &syn::ItemUse) -> Option<usize> {
```

**`method_visibility_check.rs`** (6 helpers):
```rust
// needed helper: method-file classification (no type definition)
fn is_actually_method_file(file_name: &str, parent_dir: &str, project: &Project) -> bool {
// needed helper: directory-grouped entry map
fn group_entries_by_parent_dir(entries: &[crate::entry::Entry]) -> BTreeMap<String, Vec<String>> {
// needed helper: struct name set from file listing
fn collect_struct_names(file_names: &[String]) -> HashSet<String> {
// needed helper: method-file name pattern matching
fn is_method_file(file_name: &str, struct_names: &HashSet<String>) -> Option<String> {
// needed helper: pub mod declaration check
fn declared_as_pub_mod(module_info: &ModuleInfoMap, parent_dir: &str, mod_name: &str) -> Option<PathBuf> {
// needed helper: pub use re-export check
fn reexported_in_pub_use(module_info: &ModuleInfoMap, parent_dir: &str, mod_name: &str) -> Option<PathBuf> {
```

**`module_info_build.rs`** (3 helpers):
```rust
// needed helper: mod.rs filename check
fn is_mod_rs(path: &Path) -> bool {
// needed helper: mod.rs AST parsing for declarations and re-exports
fn parse_mod_rs(content: &str) -> (Vec<ModDecl>, Vec<Reexport>) {
// needed helper: re-export path extraction from use tree
fn extract_reexport(tree: &syn::UseTree) -> Option<Reexport> {
```

**`no_positional_check.rs`** (5 helpers):
```rust
// needed helper: positional type presence check
fn has_positional_types(file: &syn::File) -> bool {
// needed helper: recursive item block scanner
fn scan_block_for_positional(items: &[syn::Item], rel_path: &Path, project: &Project, diags: &mut Vec<Diagnostic>) {
// needed helper: statement-level scanner
fn scan_stmts(stmts: &[syn::Stmt], rel_path: &Path, project: &Project, diags: &mut Vec<Diagnostic>) {
// needed helper: expression-level position access checker
fn scan_expr(expr: &syn::Expr, rel_path: &Path, project: &Project, diags: &mut Vec<Diagnostic>) {
// needed helper: macro token string scan for .0/.1 access
fn has_positional_access(content: &str) -> bool {
```

**`no_trivial_accessors_check.rs`** (4 helpers):
```rust
// needed helper: public field name collection
fn collect_pub_fields(file: &syn::File) -> HashSet<String> {
// needed helper: trivial accessor pattern detection
fn is_trivial_accessor(method: &syn::ImplItemFn, pub_fields: &HashSet<String>) -> Option<String> {
// needed helper: self.field expression extraction
fn extract_self_field(expr: &syn::Expr, pub_fields: &HashSet<String>) -> Option<String> {
// needed helper: self-reference expression check
fn is_self_ref(expr: &syn::Expr) -> bool {
```

**`single_caller_type_check.rs`** (8 helpers after extraction):
```rust
// needed helper: type definition collection across all files
fn collect_defined_types(parsed_files: &HashMap<PathBuf, syn::File>) -> Vec<(String, PathBuf)> {
// needed helper: path exemption for mod.rs/lib.rs
fn is_exempt_path(rel_path: &Path) -> bool {
// needed helper: embedded type name collection from field types
fn collect_embedded_type_names(parsed_files: &HashMap<PathBuf, syn::File>) -> HashSet<String> {
// needed helper: type path visitor
fn collect_type_paths(ty: &syn::Type, names: &mut HashSet<String>) {
// needed helper: thin delegate method presence check
fn has_thin_delegate(file: &syn::File, type_name: &str) -> bool {
// needed helper: all-delegate impl block verification
fn impl_is_all_delegate(impl_block: &syn::ItemImpl) -> bool {
// needed helper: per-file type reference collector
fn collect_file_references(parsed_files: &HashMap<PathBuf, syn::File>, defined: &HashSet<String>) -> HashMap<PathBuf, HashSet<String>> {
// needed helper: item-level reference collection with cfg(test) skip
fn collect_refs_from_items(items: &[syn::Item], defined: &HashSet<String>, found: &mut HashSet<String>) {
```

**`test_usage_check.rs`** (9 helpers after extraction):
```rust
// needed helper: opt-out comment detection in source
fn has_test_usage_opt_out(project: &Project, rel_path: &Path) -> bool {
// needed helper: file exemption rules
fn is_exempt(rel_path: &Path, file: &syn::File) -> bool {
// needed helper: pure module tree detection
fn is_pure_module_tree(file: &syn::File) -> bool {
// needed helper: type-only file detection (no non-default impls)
fn is_type_only(file: &syn::File) -> bool {
// needed helper: default-only impl block check
fn is_default_only_impl(impl_block: &syn::ItemImpl) -> bool {
// needed helper: thin-delegate-only file detection
fn is_thin_delegate_only(file: &syn::File) -> bool {
// needed helper: likely delegate impl detection
fn is_likely_delegate_impl(impl_block: &syn::ItemImpl) -> bool {
// needed helper: single-call body pattern
fn is_single_call_body(block: &syn::Block) -> bool {
// needed helper: test_usage function presence in cfg(test) module
fn has_test_usage(file: &syn::File) -> bool {
```

**`thin_delegates_check.rs`** (5 helpers after extraction):
```rust
// needed helper: primary type name from file stem
fn primary_type_name(file: &syn::File, file_stem: &str) -> Option<String> {
// needed helper: method presence check in impl block
fn has_any_method(impl_block: &syn::ItemImpl) -> bool {
// needed helper: all-methods-are-delegates check
fn is_all_delegate_methods(impl_block: &syn::ItemImpl) -> bool {
// needed helper: non-delegate method name listing
fn non_delegate_method_names(impl_block: &syn::ItemImpl) -> Vec<String> {
// needed helper: one-line body check (placeholder)
fn is_one_line_body(_method: &syn::ImplItemFn) -> bool {
```

### D4. Remove stale `// no test_usage necessary` inside test module

**`checkers/no_trivial_accessors_check.rs`** — has `// no test_usage necessary` on line 120 inside `#[cfg(test)] mod tests`. This is an error (opt-out goes at file end, not inside tests). The file already has test functions, so no opt-out is needed. Remove that line entirely.

---

## Part E: Update E015 message

**File:** `src/checkers/helper_count_check.rs`

Current message references the old blanket annotation:
```
"{} helper functions (max {}). Extract pure/reusable ones as thin delegates; keep only context-specific ones with `// needed helper:` to justify"
```

Since annotations now go on individual functions (not file-level blanket), update to:
```
"{} unannotated helper functions (max {}). Annotate context-specific helpers with `// needed helper:` on the line above each function; extract reusable ones into thin delegate files"
```

---

## Part F: Verification

After all changes:

1. **`cargo build --all-targets`** — ensure compilation succeeds
2. **`cargo clippy -- -D warnings`** — no clippy errors
3. **`cargo fmt -- --check`** — formatting clean
4. **`cargo test --all-targets`** — all tests pass
5. **`cargo run --manifest-path ../lele_lint/Cargo.toml`** (from a project dir) — lele_lint itself passes against other projects
6. **Run lele_lint on itself** — `cargo run --manifest-path ../lele_lint/Cargo.toml -- --project lele_lint` — produces zero violations

### Expected before/after test changes

**Unit tests in `helper_count_check.rs`:**
- `test_usage_counts_helpers` — needs update: `pub fn bar() {}` would no longer be counted (it's `pub`). But `fn helper_a()`, `fn helper_b()`, `fn helper_c()` are `Inherited` → still 3. Test passes unchanged.
- `test_usage_skips_impl_fns` — `fn helper_a()` is `Inherited` → 1. Test passes unchanged.

**Unit tests in `test_usage_check.rs`:**
- `test_usage_finds_test_usage` — unchanged
- `test_usage_missing` — unchanged
- New test needed: `test_usage_opt_out_only_at_end` — parses source with `// no test_usage necessary` at L1, asserts `has_test_usage_opt_out` returns `false`; with annotation at last line, asserts `true`.

**e2e tests:**
- `compliant_crate_has_no_violations` — should still pass
- `violation_crate_catches_all_errors` — should still catch all expected error codes

---

## File change summary

| Category | Files changed/created |
|----------|----------------------|
| New files | `src/common/mod.rs` + 6 utility files = 7 files |
| Linter logic | `test_usage_check.rs`, `helper_count_check.rs` |
| `lib.rs` | Add `pub mod common;` |
| Annotation moves (~55 files) | Move `// no test_usage necessary` from L1 to end |
| `// needed helper:` removal | `project.rs`, `module_info.rs`, `helper_count_check.rs`, `snake_case_files_check.rs` |
| `// needed helper:` relocation | 10 `_check.rs` files + `module_info_build.rs`: remove old blanket comment, add individual annotations |
| Stale comment removal | `no_trivial_accessors_check.rs`: remove `// no test_usage necessary` from inside test module |
| Shared util callers | `atomic_file_check.rs`, `thin_delegates_check.rs`, `constructor_no_skip_check.rs`, `test_usage_check.rs`, `single_caller_type_check.rs`: replace local fns with `common::` calls |
