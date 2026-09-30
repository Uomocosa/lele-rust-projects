use std::path::PathBuf;

use crate::index;

pub fn build_call_graph(idx: &mut index::SymbolIndex) {
    let files: Vec<PathBuf> = idx.occurrences.keys().cloned().collect();
    let mut edges: Vec<(String, String)> = Vec::new();
    for file in files {
        let items = idx.items_by_file.get(&file).cloned().unwrap_or_default();
        let occs = idx.occurrences.get(&file).cloned().unwrap_or_default();
        for occ in &occs {
            if occ.is_self {
                continue;
            }
            let Some(target) = occ.target.clone() else {
                continue;
            };
            if let Some(ci) = innermost(&items, occ.line, idx)
                && let Some(item) = idx.items.get(ci)
            {
                let caller = item.id.clone();
                if caller != target {
                    edges.push((caller, target));
                }
            }
        }
    }
    for (caller, callee) in edges {
        let outgoing = idx.callees.entry(caller.clone()).or_default();
        if !outgoing.contains(&callee) {
            outgoing.push(callee.clone());
        }
        let incoming = idx.callers.entry(callee).or_default();
        if !incoming.contains(&caller) {
            incoming.push(caller);
        }
    }
}

// needed helper: smallest enclosing item index for a line in a file
fn innermost(items: &[usize], line: usize, idx: &index::SymbolIndex) -> Option<usize> {
    let mut best: Option<usize> = None;
    let mut best_span = usize::MAX;
    for &i in items {
        let Some(item) = idx.items.get(i) else {
            continue;
        };
        if item.start_line <= line && line <= item.end_line {
            let span = item.end_line.saturating_sub(item.start_line);
            if span < best_span {
                best_span = span;
                best = Some(i);
            }
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::build_call_graph;
    use crate::index;

    #[test]
    fn test_usage() {
        let mut idx = index::SymbolIndex::default();
        let file = PathBuf::from("src/a.rs");
        idx.items.push(index::IndexItem {
            id: "a::f".to_string(),
            name: "f".to_string(),
            module: "a".to_string(),
            kind: index::ItemKind::Fn,
            file: file.clone(),
            start_line: 1,
            end_line: 3,
            name_line: 1,
            name_col_start: 0,
            name_col_end: 1,
            signature: "pub fn f".to_string(),
            doc: None,
            delegates_to: None,
            is_test: false,
            external: Vec::new(),
        });
        idx.items.push(index::IndexItem {
            id: "a::g".to_string(),
            name: "g".to_string(),
            module: "a".to_string(),
            kind: index::ItemKind::Fn,
            file: file.clone(),
            start_line: 5,
            end_line: 7,
            name_line: 5,
            name_col_start: 0,
            name_col_end: 1,
            signature: "pub fn g".to_string(),
            doc: None,
            delegates_to: None,
            is_test: false,
            external: Vec::new(),
        });
        idx.items_by_file.insert(file.clone(), vec![0, 1]);
        idx.occurrences.insert(
            file,
            vec![index::Occurrence {
                line: 2,
                start_col: 0,
                end_col: 1,
                target: Some("a::g".to_string()),
                is_self: false,
            }],
        );
        build_call_graph(&mut idx);
        assert_eq!(idx.callees.get("a::f"), Some(&vec!["a::g".to_string()]));
        assert_eq!(idx.callers.get("a::g"), Some(&vec!["a::f".to_string()]));
    }
}
