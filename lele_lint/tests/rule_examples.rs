use std::collections::BTreeSet;
use std::path::Path;
use std::path::PathBuf;

use lele_lint::checkers::build_checkers;
use lele_lint::config::Config;
use lele_lint::Checker;
use lele_lint::ExampleFile;
use lele_lint::Project;

const CATEGORIES: [&str; 7] = [
    "layout",
    "delegates",
    "imports",
    "types",
    "tests",
    "style",
    "config",
];

const DEFAULT_CARGO_TOML: &str = r#"[package]
name = "example"
version = "0.1.0"
edition = "2021"

[dependencies]
atomic_delegate_macros = { path = "../atomic_delegate_macros" }
derive_more = { version = "1", features = ["deref", "deref_mut"] }
thiserror = "2"
tracing = "0.1"

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
"#;

const DEFAULT_CLIPPY_TOML: &str = "allow-unwrap-in-tests = true
allow-expect-in-tests = true
allow-panic-in-tests = true
allow-indexing-slicing-in-tests = true
";

const DEFAULT_LELE_TOML: &str = "[lele.lint]\n";

fn write_file(root: &Path, rel: &str, source: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = root.join(rel);
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(path, source)?;
    Ok(())
}

fn materialize(root: &Path, files: &[ExampleFile]) -> Result<(), Box<dyn std::error::Error>> {
    write_file(root, "Cargo.toml", DEFAULT_CARGO_TOML)?;
    write_file(root, "clippy.toml", DEFAULT_CLIPPY_TOML)?;
    write_file(root, "lele.toml", DEFAULT_LELE_TOML)?;
    write_file(root, "src/lib.rs", "")?;
    for file in files {
        write_file(root, file.path, file.source)?;
    }
    Ok(())
}

fn lint(files: &[ExampleFile]) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;
    materialize(dir.path(), files)?;
    let mut project = Project::discover(Some(dir.path()), None)?;
    let config = Config::load(&project.root).unwrap_or_default();
    project.apply_layout(&config)?;
    let root: PathBuf = dir.path().to_path_buf();
    let mut lines: Vec<String> = build_checkers()
        .iter()
        .flat_map(|c| c.check(&project))
        .map(|d| {
            let file = d.file.strip_prefix(&root).unwrap_or(&d.file);
            format!("{} {}:{} {}", d.code, file.display(), d.line, d.message)
        })
        .collect();
    lines.sort();
    Ok(lines)
}

fn check_rule(checker: &dyn Checker) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut problems = Vec::new();
    let code = checker.code();
    let doc = checker.doc();
    if !CATEGORIES.contains(&doc.category) {
        problems.push(format!("{code}: unknown category `{}`", doc.category));
    }
    if doc.summary.is_empty() || doc.why.is_empty() {
        problems.push(format!("{code}: summary and why must not be empty"));
    }
    if doc.bad.is_empty() || doc.good.is_empty() {
        problems.push(format!("{code}: needs at least one bad and one good file"));
        return Ok(problems);
    }
    let bad = lint(doc.bad)?;
    if !bad.iter().any(|line| line.starts_with(code)) {
        problems.push(format!(
            "{code}: bad example does not trigger {code}; got:\n  {}",
            bad.join("\n  ")
        ));
    }
    let good = lint(doc.good)?;
    if !good.is_empty() {
        problems.push(format!(
            "{code}: good example must be clean; got:\n  {}",
            good.join("\n  ")
        ));
    }
    Ok(problems)
}

/// Every rule's bad example triggers its own code and its good example lints clean.
#[test]
fn every_rule_example_is_correct() -> Result<(), Box<dyn std::error::Error>> {
    let mut problems = Vec::new();
    for checker in build_checkers() {
        problems.extend(check_rule(checker.as_ref())?);
    }
    if problems.is_empty() {
        return Ok(());
    }
    Err(problems.join("\n").into())
}

/// Every file in `src/checkers/` is registered in `build_checkers`, and codes are unique.
#[test]
fn every_checker_file_is_registered() -> Result<(), Box<dyn std::error::Error>> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/checkers");
    let mut files = BTreeSet::new();
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        if stem != "mod" && stem != "build_checkers" {
            files.insert(stem.to_string());
        }
    }
    let checkers = build_checkers();
    let names: BTreeSet<String> = checkers.iter().map(|c| c.name().to_string()).collect();
    let codes: BTreeSet<&str> = checkers.iter().map(|c| c.code()).collect();
    if files != names {
        return Err(format!("checker files {files:?} != registered {names:?}").into());
    }
    if codes.len() != checkers.len() {
        return Err("duplicate checker codes".into());
    }
    Ok(())
}
