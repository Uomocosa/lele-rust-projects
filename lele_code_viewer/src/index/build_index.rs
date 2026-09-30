use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;

use lele_lint::Project;

use crate::Error;
use crate::index;

pub fn build_index(crate_dir: &Path) -> Result<index::SymbolIndex, Error> {
    let crate_dir = crate_dir.canonicalize()?;
    let mut project = Project::discover(Some(&crate_dir), None)?;
    let config = lele_lint::Config::load(&project.root)?;
    project.apply_layout(&config)?;

    let mut idx = index::SymbolIndex {
        root: project.root.clone(),
        crate_name: index::crate_name_of(&project.root),
        ..index::SymbolIndex::default()
    };
    idx.files = read_sources(&project);
    let crate_deps = index::crate_externs(&project.root);

    for (rel, file) in &project.parsed_files {
        let key = PathBuf::from("src").join(rel);
        let text = idx.files.get(&key).map_or("", String::as_str);
        idx.items
            .extend(index::collect_file_items(&key, text, file, &crate_deps));
    }
    for (rel, file) in &project.methods_parsed_files {
        let key = PathBuf::from("methods").join(rel);
        let text = idx.files.get(&key).map_or("", String::as_str);
        idx.items
            .extend(index::collect_file_items(&key, text, file, &crate_deps));
    }

    index_lookups(&mut idx);

    let mut imports_by_file: HashMap<PathBuf, index::Imports> = HashMap::new();
    for (rel, file) in &project.parsed_files {
        imports_by_file.insert(PathBuf::from("src").join(rel), index::collect_imports(file));
    }
    for (rel, file) in &project.methods_parsed_files {
        imports_by_file.insert(
            PathBuf::from("methods").join(rel),
            index::collect_imports(file),
        );
    }

    for (rel, file) in &project.parsed_files {
        let key = PathBuf::from("src").join(rel);
        let imports = imports_by_file.get(&key).cloned().unwrap_or_default();
        let occs = index::collect_occurrences(&key, file, &imports, &idx);
        idx.occurrences.insert(key, occs);
    }
    for (rel, file) in &project.methods_parsed_files {
        let key = PathBuf::from("methods").join(rel);
        let imports = imports_by_file.get(&key).cloned().unwrap_or_default();
        let occs = index::collect_occurrences(&key, file, &imports, &idx);
        idx.occurrences.insert(key, occs);
    }

    index::build_call_graph(&mut idx);

    let mut file_deps: HashMap<PathBuf, index::FileDeps> = HashMap::new();
    for (rel, file) in &project.parsed_files {
        file_deps.insert(PathBuf::from("src").join(rel), index::collect_deps(file));
    }
    for (rel, file) in &project.methods_parsed_files {
        file_deps.insert(
            PathBuf::from("methods").join(rel),
            index::collect_deps(file),
        );
    }
    index::build_module_graph(&mut idx, &file_deps);
    index::build_item_graph(&mut idx);

    idx.rust_files = idx.files.keys().cloned().collect();
    idx.rust_files.sort();
    idx.markdown_files = find_markdown(&project.root);
    idx.extra_files = find_extra_files(&project.root);
    Ok(idx)
}

// needed helper: read every rust source keyed by src/- or methods/-prefixed relative path
fn read_sources(project: &Project) -> HashMap<PathBuf, String> {
    let mut files = HashMap::new();
    for entry in &project.entries {
        if entry.absolute_path.is_file()
            && let Ok(text) = std::fs::read_to_string(&entry.absolute_path)
        {
            files.insert(PathBuf::from("src").join(&entry.relative_path), text);
        }
    }
    for entry in &project.methods_entries {
        if entry.absolute_path.is_file()
            && let Ok(text) = std::fs::read_to_string(&entry.absolute_path)
        {
            files.insert(PathBuf::from("methods").join(&entry.relative_path), text);
        }
    }
    files
}

// needed helper: build id/name/module/file lookups plus enclosing-item lists
fn index_lookups(idx: &mut index::SymbolIndex) {
    let items = std::mem::take(&mut idx.items);
    for (i, item) in items.iter().enumerate() {
        idx.by_id.insert(item.id.clone(), i);
        idx.by_name.entry(item.name.clone()).or_default().push(i);
        let module = item.module.clone();
        idx.by_module_name
            .entry((module.clone(), item.name.clone()))
            .or_default()
            .push(i);
        if let Some(leaf) = module.rsplit("::").next()
            && leaf != module
        {
            idx.by_module_name
                .entry((leaf.to_string(), item.name.clone()))
                .or_default()
                .push(i);
        }
        if let Some(first) = module.split("::").next()
            && first != module
            && !first.is_empty()
        {
            idx.by_module_name
                .entry((first.to_string(), item.name.clone()))
                .or_default()
                .push(i);
        }
        idx.items_by_file
            .entry(item.file.clone())
            .or_default()
            .push(i);
    }
    idx.items = items;
}

// needed helper: collect markdown files under the crate root (skipping build dirs)
fn find_markdown(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let walker = walkdir::WalkDir::new(root)
        .into_iter()
        .filter_entry(|entry| {
            let name = entry.file_name().to_string_lossy();
            !(entry.file_type().is_dir()
                && matches!(
                    name.as_ref(),
                    "target" | ".git" | ".devenv" | "node_modules"
                ))
        });
    for entry in walker.flatten() {
        if entry.file_type().is_file()
            && entry.path().extension().is_some_and(|e| e == "md")
            && let Ok(rel) = entry.path().strip_prefix(root)
        {
            out.push(rel.to_path_buf());
        }
    }
    out.sort();
    out
}

// needed helper: collect non-rust, non-markdown files under the crate root (skipping build dirs)
fn find_extra_files(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let walker = walkdir::WalkDir::new(root)
        .into_iter()
        .filter_entry(|entry| {
            let name = entry.file_name().to_string_lossy();
            !(entry.file_type().is_dir()
                && matches!(
                    name.as_ref(),
                    "target" | ".git" | ".devenv" | "node_modules" | "__OLD__" | ".freenet"
                ))
        });
    for entry in walker.flatten() {
        if entry.file_type().is_file()
            && let Ok(rel) = entry.path().strip_prefix(root)
        {
            let is_code = entry
                .path()
                .extension()
                .is_some_and(|e| e == "rs" || e == "md");
            if !is_code {
                out.push(rel.to_path_buf());
            }
        }
    }
    out.sort();
    out
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::build_index;

    #[test]
    fn test_usage() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fs::write(
            root.join("Cargo.toml"),
            "[package]\nname = \"demo\"\nversion = \"0.0.0\"\n",
        )
        .unwrap();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/lib.rs"), "pub mod foo;\npub fn top() {}\n").unwrap();
        fs::write(
            root.join("src/foo.rs"),
            "pub struct Bar;\nimpl Bar { pub fn new() -> Self { Self } }\n",
        )
        .unwrap();
        fs::write(root.join("README.md"), "# Demo\n").unwrap();
        let idx = build_index(root).unwrap();
        assert_eq!(idx.crate_name, "demo");
        assert!(idx.items.iter().any(|i| i.id == "foo::Bar"));
        assert!(idx.items.iter().any(|i| i.id == "foo::Bar::new"));
        assert!(
            idx.markdown_files
                .contains(&std::path::PathBuf::from("README.md"))
        );
        assert!(
            idx.extra_files
                .contains(&std::path::PathBuf::from("Cargo.toml"))
        );
        assert!(
            !idx.extra_files
                .iter()
                .any(|f| f.extension().is_some_and(|e| e == "rs" || e == "md"))
        );
    }
}
