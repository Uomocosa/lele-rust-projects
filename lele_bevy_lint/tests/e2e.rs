use std::path::PathBuf;

use lele_bevy_lint::checkers::build_checkers;
use lele_lint::Config;
use lele_lint::Diagnostic;
use lele_lint::Project;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("test_fixtures")
        .join(name)
}

fn run_checkers(path: &str) -> Vec<Diagnostic> {
    let discovered = Project::discover(Some(&fixture_path(path)), None);
    assert!(discovered.is_ok(), "fixture discover failed for {path}");
    let Ok(p) = discovered else {
        return Vec::new();
    };
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
        "E005", // bevy_export
        "E008", // bevy_folder
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
