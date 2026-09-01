use std::path::Path;

use serde::Deserialize;

use crate::taxonomy_ctx::TaxonomyCtx;

#[derive(Deserialize, Debug, Default)]
struct LeleToml {
    honesty: Option<HonestySection>,
}

#[derive(Deserialize, Debug, Default)]
struct HonestySection {
    #[serde(default)]
    declared_honest: Vec<String>,
    #[serde(default)]
    declared_dishonest: Vec<String>,
    #[serde(default)]
    honesty_depth: Option<usize>,
    #[serde(default)]
    entry_allowlist: Option<Vec<String>>,
}

pub fn load(manifest_path: Option<&Path>) -> TaxonomyCtx {
    let start = manifest_path
        .and_then(|p| p.parent())
        .unwrap_or_else(|| Path::new("."));
    let mut dir = start.to_path_buf();
    loop {
        let candidate = dir.join("lele.toml");
        if candidate.exists() {
            if let Ok(content) = std::fs::read_to_string(&candidate) {
                if let Ok(parsed) = toml::from_str::<LeleToml>(&content) {
                    let h = parsed.honesty.unwrap_or_default();
                    let mut ctx = TaxonomyCtx {
                        declared_honest: h.declared_honest.into_iter().collect(),
                        declared_dishonest: h.declared_dishonest.into_iter().collect(),
                        ..Default::default()
                    };
                    if let Some(d) = h.honesty_depth {
                        ctx.honesty_depth = d;
                    }
                    if let Some(allow) = h.entry_allowlist {
                        ctx.entry_allowlist = normalize_allowlist(allow);
                    }
                    return ctx;
                }
            }
            break;
        }
        match dir.parent() {
            Some(p) => dir = p.to_path_buf(),
            None => break,
        }
        if dir == Path::new("/") {
            break;
        }
    }
    normalize_ctx(TaxonomyCtx::default())
}

fn normalize_allowlist(input: Vec<String>) -> Vec<String> {
    use std::collections::HashSet;
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for p in input {
        let n = p.strip_prefix("src/").unwrap_or(p.as_str()).to_string();
        if seen.insert(n.clone()) {
            out.push(n);
        }
    }
    out
}

fn normalize_ctx(mut ctx: TaxonomyCtx) -> TaxonomyCtx {
    ctx.entry_allowlist = normalize_allowlist(ctx.entry_allowlist);
    ctx
}

#[cfg(test)]
mod tests {
    use super::load;
    use std::path::PathBuf;

    #[test]
    fn test_usage() {
        let ctx = load(None);
        assert_eq!(ctx.honesty_depth, 1);

        let tmp = tempfile::tempdir().unwrap();
        let manifest = tmp.path().join("Cargo.toml");
        std::fs::write(&manifest, "").unwrap();
        let lele = tmp.path().join("lele.toml");
        std::fs::write(
            &lele,
            "[honesty]\nhonesty_depth = 2\nentry_allowlist = [\"src/main.rs\"]\n",
        )
        .unwrap();
        let ctx2 = load(Some(&manifest));
        assert_eq!(ctx2.honesty_depth, 2);
        assert_eq!(ctx2.entry_allowlist, vec!["main.rs".to_string()]);
        let _ = PathBuf::from("x");
    }
}
