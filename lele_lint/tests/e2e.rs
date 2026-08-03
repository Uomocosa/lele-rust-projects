use std::path::PathBuf;

use lele_lint::checkers::build_checkers;
use lele_lint::config::Config;
use lele_lint::diagnostic::Diagnostic;
use lele_lint::project::Project;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("test_fixtures")
        .join(name)
}

fn run_checkers(path: &str) -> Vec<Diagnostic> {
    let p = Project::discover(Some(&fixture_path(path))).unwrap();
    let config = Config::load(&p.root).unwrap_or_default();
    let checkers = build_checkers(&config);
    checkers.iter().flat_map(|c| c.check(&p)).collect()
}

#[test]
fn compliant_crate_has_no_violations() {
    let diags = run_checkers("compliant_crate");
    assert!(
        diags.is_empty(),
        "expected no violations in compliant crate, got {diags:?}",
        diags = diags
    );
}

#[test]
fn violation_crate_catches_all_errors() {
    let diags = run_checkers("violation_crate");
    let codes: Vec<&str> = diags.iter().map(|d| d.code.as_str()).collect();

    let expected = [
        "E002", // snake_case_files
        "E003", // method_visibility
        "E004", // no_cross_domain_reexport
        "E006", // test_usage
        "E007", // test_inline
        "E009", // no_positional
        "E010", // no_trivial_accessors
        "E011", // domain_import
        "E012", // thin_delegates
        "E013", // constructor_no_skip
    ];

    for code in &expected {
        assert!(
            codes.contains(code),
            "expected {code} in violation crate, got codes: {codes:?}",
            code = code,
            codes = codes,
        );
    }
}
