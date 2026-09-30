use crate::index;

pub fn resolve(
    segments: &[String],
    imports: &index::Imports,
    idx: &index::SymbolIndex,
) -> Option<String> {
    if segments.is_empty() {
        return None;
    }
    let segs = expand(segments, imports);
    let joined = segs.join("::");
    if let Some(id) = idx
        .by_id
        .get(&joined)
        .and_then(|&i| idx.items.get(i))
        .map(|item| item.id.clone())
    {
        return Some(id);
    }
    let name = segs.last()?.clone();
    if segs.len() >= 2 {
        let key = (
            segs.get(segs.len().saturating_sub(2))?.clone(),
            name.clone(),
        );
        if let Some(id) = lookup(&key, idx) {
            return Some(id);
        }
    }
    if let Some(first) = segs.first()
        && let Some(id) = lookup(&(first.clone(), name.clone()), idx)
    {
        return Some(id);
    }
    if let Some(v) = idx.by_name.get(&name) {
        return unique(v, idx);
    }
    None
}

// needed helper: expand an aliased first segment into its canonical path
fn expand(segments: &[String], imports: &index::Imports) -> Vec<String> {
    let mut out = segments.to_vec();
    if let Some(first) = out.first()
        && let Some(canonical) = imports.get(first)
    {
        let mut replacement: Vec<String> = canonical.split("::").map(str::to_string).collect();
        out.remove(0);
        replacement.extend(out);
        return replacement;
    }
    out
}

// needed helper: resolve a (module, name) lookup entry to a unique item id
fn lookup(key: &(String, String), idx: &index::SymbolIndex) -> Option<String> {
    let v = idx.by_module_name.get(key)?;
    unique(v, idx)
}

// needed helper: only resolve when exactly one candidate exists
fn unique(v: &[usize], idx: &index::SymbolIndex) -> Option<String> {
    if v.len() == 1 {
        v.first()
            .and_then(|&i| idx.items.get(i))
            .map(|item| item.id.clone())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::resolve;
    use crate::index;

    #[test]
    fn test_usage() {
        let mut idx = index::SymbolIndex::default();
        idx.items.push(index::IndexItem {
            id: "clicker::config::Config".to_string(),
            name: "Config".to_string(),
            module: "clicker::config".to_string(),
            kind: index::ItemKind::Struct,
            file: PathBuf::from("src/clicker/config.rs"),
            start_line: 1,
            end_line: 1,
            name_line: 1,
            name_col_start: 0,
            name_col_end: 6,
            signature: "pub struct Config".to_string(),
            doc: None,
            delegates_to: None,
            is_test: false,
            external: Vec::new(),
        });
        idx.by_id.insert("clicker::config::Config".to_string(), 0);
        idx.by_name.insert("Config".to_string(), vec![0]);
        idx.by_module_name
            .insert(("config".to_string(), "Config".to_string()), vec![0]);
        idx.by_module_name
            .insert(("clicker".to_string(), "Config".to_string()), vec![0]);
        let imports = index::Imports::default();
        let segs = vec!["clicker".to_string(), "Config".to_string()];
        assert_eq!(
            resolve(&segs, &imports, &idx).as_deref(),
            Some("clicker::config::Config")
        );
    }
}
