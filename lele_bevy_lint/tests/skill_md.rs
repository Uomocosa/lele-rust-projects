use std::path::PathBuf;

use lele_bevy_lint::skill_markdown;

/// `.opencode/skills/bevy-ui-preview/SKILL.md` is generated from the rule docs
/// and must not drift from them. Run with `LELE_BLESS=1` to regenerate it.
#[test]
fn skill_md_is_up_to_date() -> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../.opencode/skills/bevy-ui-preview/SKILL.md");
    let actual = skill_markdown();
    if std::env::var_os("LELE_BLESS").is_some() {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        std::fs::write(&path, &actual)?;
        return Ok(());
    }
    let expected = std::fs::read_to_string(&path).unwrap_or_default();
    if expected == actual {
        return Ok(());
    }
    Err(
        "SKILL.md is stale — regenerate with `LELE_BLESS=1 cargo nextest run --test skill_md`"
            .into(),
    )
}
