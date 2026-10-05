use std::path::PathBuf;

use lele_bevy_lint::checkers::build_checkers;
use lele_lint::rules_markdown;

/// `RULES.md` is generated from the rule docs and must not drift from them.
/// Run with `LELE_BLESS=1` to regenerate it.
#[test]
fn rules_md_is_up_to_date() -> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("RULES.md");
    let actual = rules_markdown(&build_checkers());
    if std::env::var_os("LELE_BLESS").is_some() {
        std::fs::write(&path, &actual)?;
        return Ok(());
    }
    let expected = std::fs::read_to_string(&path).unwrap_or_default();
    if expected == actual {
        return Ok(());
    }
    Err(
        "RULES.md is stale — regenerate with `LELE_BLESS=1 cargo nextest run --test rules_md`"
            .into(),
    )
}
