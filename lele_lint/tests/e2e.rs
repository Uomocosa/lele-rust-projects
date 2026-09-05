use std::path::PathBuf;

use lele_lint::checkers::build_checkers;
use lele_lint::config::Config;
use lele_lint::Diagnostic;
use lele_lint::Project;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("test_fixtures")
        .join(name)
}

fn run_checkers(path: &str) -> Result<Vec<Diagnostic>, Box<dyn std::error::Error>> {
    let p = Project::discover(Some(&fixture_path(path)), None)?;
    let config = Config::load(&p.root).unwrap_or_default();
    let checkers = build_checkers(&config);
    Ok(checkers.iter().flat_map(|c| c.check(&p)).collect())
}

#[test]
fn compliant_crate_has_no_violations() -> Result<(), Box<dyn std::error::Error>> {
    let diags = run_checkers("compliant_crate")?;
    if !diags.is_empty() {
        return Err(format!("expected no violations in compliant crate, got {diags:?}").into());
    }
    Ok(())
}

#[test]
fn violation_crate_catches_all_errors() -> Result<(), Box<dyn std::error::Error>> {
    let diags = run_checkers("violation_crate")?;
    let codes: Vec<&str> = diags.iter().map(|d| d.code.as_str()).collect();

    let expected = [
        "E001", // atomic_file (orphan method file has no parent type)
        "E002", // snake_case_files
        "E003", // method_visibility
        "E004", // no_cross_domain_reexport
        "E006", // test_usage
        "E007", // test_inline
        "E009", // no_positional
        "E010", // no_trivial_accessors
        "E011", // domain_import
        "E012", // atomic_delegates
        "E013", // constructor_no_skip
        "E016", // single_caller_type
        "E017", // method_file_co_location (orphan method file)
        "E018", // single_field_newtype
        "E019", // mod_rs_purity
        "E020", // no_crate_paths
        "E024", // root_reexport
        "E025", // no_stuttered_path
        "E027", // no_stuttered_type
    ];

    for code in expected {
        if !codes.contains(&code) {
            return Err(format!("expected {code} in violation crate, got codes: {codes:?}").into());
        }
    }
    Ok(())
}
