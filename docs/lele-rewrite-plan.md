# Plan: lele_lint rewrite, lele-syntax-rs rewrite, reliable function taxonomy

Audience: the agent that executes this plan. Read all of it before touching code.
Workspace root: `rust_projects/projects/` (paths below are relative to it).
Branch: `refactor/discovery-cleanup`.

---

## 1. Decisions already made by the user (binding — do not re-open)

| Topic | Decision |
|---|---|
| Rule severity | Every rule is always on and always an **error**. No warnings, no per-rule switches, no `off`. |
| `methods/`, `__basic__/`, `bevy_systems/` | Keep exactly as they are. They are deliberate. |
| Single-method structs, single-implementor traits | Fine. Never add a rule against them. A trait states a contract. |
| "Rule of three" | **Not** adopted. The user's rule is "one function, one job" (see §5.2). |
| Special comments | Keep `// needed helper: <why>` and `// no test_usage necessary` as they are. |
| Rule codes | Keep the current numbers. E005, E008, E014 are retired gaps. New rules continue at E035. |
| Golden files | Plain `.txt` files, no `insta`. |
| Validated id newtypes (E018 + private field + `FromStr`) | Not now. Documentation only (§5.2, "Parse, don't validate"). |
| Which crates to lint | **Only `lele_lint` and `freenet_libp2p_bevy_plugin`.** The other crates are known to be far behind; do not run lele_lint on them, do not fix them. |
| `bool` parameters | Allowed (`set_visible(visible: bool)` is data). No flag-argument rule. |

## 2. Ground rules for the executing agent

- Read the crate's `devenv.nix` first. Run checks **only** via `devenv tasks run <task> 2>&1`
  (`lele:fmt`, `lele:clippy`, `lele:nextest`, `lele:lint`). Never pipe the output (`| tail` etc.);
  redirect to a file if you need to grep it. Never run `bacon`.
- `devenv` lives in `~/.nix-profile/bin` (prepend it to `PATH` if `devenv` is not found).
- To *format* (not check) use `devenv shell -- cargo fmt`.
- Never add `#[allow(...)]` / `#[expect(...)]`. If clippy objects, rewrite the code; if you
  cannot, stop and ask the user.
- No comments in `src/` or `methods/` except the two sanctioned markers.
- Commit **with** hooks (no `--no-verify`) after each phase; hooks run fmt, clippy, lele_lint,
  taxonomy and docs checks for `lele_lint`. Conventional commit messages.
- After every phase: `lele:fmt`, `lele:clippy`, `lele:nextest`, `lele:lint` in `lele_lint/`
  must pass, and `devenv tasks run lele:lint 2>&1` in `freenet_libp2p_bevy_plugin/` must
  still report 0 diagnostics.
- Golden files: `LELE_BLESS=1` + the test re-writes `lele_lint/tests/golden/*.txt` and
  `lele_lint/RULES.md`. Only bless when the change in output is intended, and say so in the
  commit message.
- Files under `__OLD__/` are git-ignored and retired. Never edit them.

## 3. Where things stand

Done and committed:

| Commit | Phase | Content |
|---|---|---|
| `b5f4d19` | P0 | `tests/golden.rs` + `tests/golden/*.txt`: exact diagnostics for the 4 fixtures. |
| `8ca153f` | P1 | Every checker has `pub const DOC: RuleDoc` (category, summary, why, bad/good example crates). `--explain <CODE|name>`, `--rules-md`, generated `lele_lint/RULES.md`, `tests/rule_examples.rs` (bad reports its code, good is clean, all checker files registered, codes unique), `tests/rules_md.rs` (RULES.md not stale). `Severity` removed. |
| `de21c13` | P2 | Strict `lele.toml`: unknown keys under `[lele]` / `[lele.lint]` are errors; a broken `lele.toml` stops lele_lint. Dead `[lele.lint.checkers]` removed from every crate. |

**Saved work in progress (P3 step 1):** the config schema for the new rules, stored as a patch —
`docs/lele-rewrite-p3-schema.patch` (made against `de21c13`; it compiles). Start P3 with
`git apply docs/lele-rewrite-p3-schema.patch`, then delete the patch file in the P3 commit. It contains:

- `src/__basic__/structs.rs`: `LeleSection { lint, vocabulary: Vec<VocabularyEntry>, boundary: Vec<BoundaryEntry>, enforce_config }`,
  `VocabularyEntry { name, meaning, banned }`, `BoundaryEntry { name, why, folders, cannot_use }`, all `deny_unknown_fields`.
- `Config` is now `Config(pub Option<LeleSection>)`; `load.rs`, `dunder.rs`, `apply_layout.rs` adapted.
- `Project` gained `vocabulary` and `boundaries` (filled in `apply_layout`).
- **Do not commit it alone**: it parses config that nothing enforces yet. Commit it together with E035/E036.
- `BoundaryEntry` has **no** `require` field in the patch; it is added by the taxonomy plan (see §6).

How a rule is built (follow the existing pattern exactly):

1. `src/checkers/<name>.rs`: unit struct, `NAME`, `CODE`, `DOC`, `impl Checker` with
   `#[atomic_delegate(<Struct>)] fn check(...) {}` and `register`.
2. `methods/<name>/check.rs`: `pub fn check(_self: &checkers::<name>::<Struct>, project: &Project) -> Vec<Diagnostic>`,
   private helpers each preceded by `// needed helper: <why>`, and a `#[cfg(test)]` `test_usage`.
3. `methods/<name>/mod.rs` and `methods/mod.rs`: regenerate with `lele_lint --sync-methods`
   (`cargo run -- --sync-methods` inside `lele_lint/`), do not hand-write.
4. Add `pub(crate) mod <name>;` to `src/checkers/mod.rs` and a `register` line in `build_checkers.rs`.
5. `DOC.bad` must report the code, `DOC.good` must lint completely clean; `tests/rule_examples.rs` enforces it.
   Default example files (added unless the example provides them): empty `src/lib.rs`, a compliant
   `Cargo.toml`, `clippy.toml`, `lele.toml = "[lele.lint]"`.
6. Bless `RULES.md`.

## 4. P3 — new rules

### P3a — E035 `vocabulary` (category `style`)

Config (per crate, in `lele.toml`):

```toml
[[lele.vocabulary]]
name    = "directory"
meaning = "The shared list of open rooms, stored in the Freenet contract."
banned  = ["board", "catalogue", "catalog", "slots", "index"]
```

Behaviour:

- **What is checked:** names *we* declare, never names we only reference (an external `IndexMap` must not trigger "index"):
  - file and folder names under `src/` and `methods/` (each path component, `.rs` stripped);
  - identifiers of: `fn`, `struct`, `enum`, enum variants, named fields, `const`, `static`, `type`, `trait`, `mod`,
    inherent-impl methods and trait-*definition* methods, function parameters and `let`/pattern bindings (`syn::visit::visit_pat_ident`).
  - Skip method names inside `impl Trait for X` blocks (the trait decides those names).
  - Include `#[cfg(test)]` code (tests use the same language).
- **Word matching:** split identifiers into lowercase words on `_`, `-` and camelCase / acronym boundaries
  (`run_board` → `run, board`; `BoardState` → `board, state`; `HTTPServer` → `http, server`).
  A banned term may be several words (`"lobby_room"`); it matches when its words appear **contiguously**.
  `keyboard` must **not** match `board`. Put the splitter in `src/common/split_words.rs` with its own tests.
- **Config validation (also E035, reported on `lele.toml` line 1):** empty `banned`; `name` itself banned;
  the same banned word under two names.
- **Message:** `` `run_board` uses banned word `board` — use `directory` (directory: The shared list of open rooms, stored in the Freenet contract.) ``
  with the identifier's real line (`span().start().line`; `proc-macro2` already has `span-locations`).
- **Examples:** bad = `src/merge_board.rs` with `pub fn merge_board`; good = `merge_directory`. Both need the `lele.toml` above.
- **Tests:** splitter cases (snake, camel, acronym, digits), `keyboard` vs `board`, multi-word term, trait-impl method skipped, file-name hit, config validation.

### P3b — E036 `boundary_imports` (category `imports`)

Config:

```toml
[[lele.boundary]]
name       = "session is pure protocol logic"
why        = "Session is driven by run.rs; it receives `now` and messages as arguments and returns actions."
folders    = ["src/discovery/session"]
cannot_use = ["tokio", "libp2p", "bevy", "std::net", "std::fs", "std::time"]
```

Behaviour:

- **Which files are inside a boundary:** a file whose crate-relative path (`src/<rel>`) starts with one of `folders`
  (component-wise, `Path::starts_with`). A `methods/<type_snake>/*.rs` file is inside when the type it belongs to is
  defined in a file inside the boundary (map type → file with `common::primary_type_name` + `to_snake_case`), or when
  `methods/...` itself is listed in `folders`.
- **What counts as a use:** every `use` tree at any depth (flatten groups; record renames), and every `syn::Path`
  (visit_path; this also covers macro paths like `tokio::select!`).
  - Build an alias map from the file's `use` items (last segment or rename → full path) and expand a path whose first
    segment is an alias (`use std::time; ... time::Instant::now()` → `std::time::Instant::now`).
  - Ignore paths starting with `crate`, `self`, `super`.
- **Hit:** the (expanded) path starts with a forbidden path (segment-wise). For a glob import, also a hit when the
  forbidden path starts with the glob's base (`use std::time::*;` with `std::time::Instant` forbidden).
- **Config validation (also E036, on `lele.toml`):** a folder that does not exist; empty `folders` or `cannot_use`.
- **Message:** `` `tokio::sync::mpsc` is not allowed in boundary "session is pure protocol logic" (src/discovery/session) — <why> ``
  with the real line. One diagnostic per (file, line, path).
- **Known limit (document it in the rule's `why` or RATIONALE):** a crate-local function that does I/O
  (e.g. `discovery::now_epoch()`) passes this rule. Catching that is the taxonomy's job (§6).
- **Examples:** bad = `src/session/expire.rs` calling `std::time::SystemTime::now()`; good = same function taking `now: u64`.

### P3c — adopt in the plugin

Add to `freenet_libp2p_bevy_plugin/lele.toml`:
- vocabulary entries `directory` (banned `board`, `catalogue`, `catalog`, `slots`) and `room`
  (ask the user for `meaning` texts and further banned words before committing);
- the `session` boundary above.

Run `lele:lint` there. If the boundary reports violations, **do not weaken it and do not refactor the plugin**:
commit only the vocabulary, report the boundary violations to the user (the sans-IO refactor is separate work).

## 5. P4 — rewrite the `lele-syntax-rs` skill

The skill lives outside the repo: `~/.config/opencode/skills/lele-syntax-rs/` (not under git; edits are local).
Also touch `~/.config/opencode/skills/lele-rs/SKILL.md` (index) and
`~/.config/opencode/skills/lele-rs/references/lele-rust-config/lele.toml` (template).

### 5.1 `SKILL.md` (target ~80–100 lines, now 311)

Keep only what an agent needs before writing code and what lele_lint cannot tell it:
1. Mental model: domain folders; atomic files; struct file + `#[atomic_delegates]` shells + bodies in
   `methods/<type>/<method>.rs`; `__basic__/` for behavior-free types; `bevy_systems/`; import the domain
   (`use crate::stock;` → `stock::Item`).
2. How to work with the linter: run `devenv tasks run lele:lint 2>&1`; every diagnostic has a code;
   `lele_lint --explain E0xx` prints the rule with bad/good examples; the full list is `../lele_lint/RULES.md`
   (generated — never copy rules into the skill).
3. The `#[allow]` gate (unchanged policy).
4. What the linter cannot check → `references/RATIONALE.md`.

Remove: the duplicated build routine (§10, lives in `lele-rs` and `AGENTS.md`), the E021/E022 details (§12, now in
`RULES.md`), the checker-bug note in §8, the per-rule prose that `RULES.md` now covers.
Delete `references/EXAMPLES.md` (superseded by the rule examples) after checking nothing unique is lost.

### 5.2 `references/RATIONALE.md` (new) — the rules that are judgement, not lint

One section each: a 2–3 line rule, a *general* example (not tied to this repo unless noted), which lele rule
enforces part of it (or "judgement only"), and links. **Verify every link with a web search before writing it.**

1. **One function, one job** (the user's "rule of one"). When a piece of code does one nameable thing, extract it
   into a function with that name and a colocated unit test; the name, arguments and return type are the
   documentation (hence no comments). If two functions differ in one inner step, pass that step as a parameter
   (`impl Fn`). Limits: pass behaviour not a flag-enum; at most one behaviour parameter (more → separate functions,
   or a trait if the behaviours belong together); the passed function is a named fn with its own test; don't merge
   when the shared part is ≤2 statements; undo the merge when callers bend to fit it.
   Refs: Fowler *Refactoring* — Extract Function, Parameterize Function; Fowler bliki "FunctionLength";
   Rust Book ch. 13 (closures, `impl Fn`); Sandi Metz "The Wrong Abstraction" (for the last limit only).
   **Do not** present "rule of three" / AHA as a rule.
2. **Ubiquitous language** — why E035 exists; how to pick `name` and write `meaning`. Ref: Fowler "UbiquitousLanguage".
3. **Functional core, imperative shell / Sans-IO / ports and adapters** — why E036 exists; core folders vs adapter
   folders (adapters are named after the technology they talk to). Refs: Gary Bernhardt "Boundaries",
   "Functional Core, Imperative Shell"; Firezone "Sans-IO"; sans-io.readthedocs.io; Cockburn "Hexagonal architecture".
4. **Group by feature** — domain folders group by feature; `methods/`, `__basic__/`, `bevy_systems/` are the three
   deliberate exceptions and why (hide bodies until needed; small types don't need a file and a test each; systems
   are found in one place). Ref: "Screaming Architecture".
5. **Parse, don't validate** — *when* a value needs validation, make an invalid one unrepresentable: private field +
   `FromStr`/`TryFrom`; general examples (email, port number, non-empty string). Not enforced. Ref: Alexis King.
6. **E2E tests: Screenplay pattern** — app handles with plain `async` methods, the test owns the timeouts,
   `run.step(name, fut)`; and **turmoil** for in-process tests of a sans-IO core. Refs: Serenity/JS Screenplay
   handbook, turmoil docs + "Announcing turmoil".
7. **Test vocabulary** — unit test (Rust's name for inline tests), usage test (`test_usage`), doctest,
   integration test, smoke test, characterization test, snapshot/golden test, property-based test — one line each.
8. Move the old §14 "Dummy `Default` for process handles" here.

### 5.3 Index and template

- `lele-rs/SKILL.md`: point "rules" to `lele-syntax-rs` + `RULES.md`; mention `--explain`.
- Template `lele.toml`: keep it minimal — `[lele.lint]` plus one commented-out example of
  `[[lele.vocabulary]]` and `[[lele.boundary]]` (TOML comments are fine; the no-comments rule is for Rust code).

## 6. Reliable function taxonomy (`lele_function_taxonomy`) — separate plan, runs FIRST

The taxonomy rewrite has its own plan (`docs/taxonomy-mir-plan.md`, written after this one) and the user
executes it **before** this plan. Decisions taken (binding):

| Question | Decision |
|---|---|
| Approach | **B: rustc driver reading MIR**, inside the existing crate (run as `RUSTC_WORKSPACE_WRAPPER`, pinned nightly + `rustc-dev`). |
| Model | **Boundaries only.** `require = "honest"` on `[[lele.boundary]]` folders. Remove `honesty_depth` and `entry_allowlist`. `[honesty] declared_dishonest` / `declared_honest` stay as the leaf lists. |
| Unknown external functions | **Assumed honest** (fewer false alarms). Only listed I/O roots are dishonest. |
| `&self` + interior mutability | See the MIR plan (decision recorded there). |

Findings that motivated it (verified by reading the code): the crate never uses rustc (no `extern crate rustc_*`);
classification is substring matching (`'static` ⇒ "dishonest"; aliases escape); only free `fn`s in `src/` are read
(no methods, closures, `methods/`); calls keyed by bare name; method-call syntax ignored; one-pass, order-dependent
propagation; diagnostics at line 1; code `TAX001`. The `taxonomy_check` hook therefore almost never fails.

**Coordination with this plan:** `[[lele.boundary]]` is shared by both tools, and lele_lint's `lele.toml` schema is
strict. Because the MIR plan lands first, it must apply `docs/lele-rewrite-p3-schema.patch` itself and add
`require: Option<Requirement>` (enum `Requirement { Honest }`, serde lowercase) to lele_lint's `BoundaryEntry`,
otherwise `lele.toml` files using `require` are rejected. If it does, P3 here starts from that state instead of the
patch (skip `git apply`). Check `git log` first so the patch is never applied twice.

## 7. P5 (later, not part of this plan)

Merge overlapping rules once `tests/rule_examples.rs` shows real double reports:
imports (E011/E020/E024/E025/E033/E004), placement (E001/E017/E029/E030), delegates (E003/E012/E013/E030/E032),
struct shape (E009/E018/E028). Keep old codes as aliases in `--explain`.

## 8. Known issues outside this plan (report, don't fix)

- `lele_bevy_lint` does not compile against current `lele_lint` (it imports `lele_lint::diagnostic::*`,
  `lele_lint::severity::*` paths that no longer exist, and now lacks `Checker::doc`).
- The other crates are behind on lint (user knows).
