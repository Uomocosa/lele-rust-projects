use std::sync::Arc;

use crate::Error;
use crate::index;
use crate::project;

pub fn index_for(
    registry: &project::Registry,
    item: &project::ProjectRef,
) -> Result<Arc<index::SymbolIndex>, Error> {
    if let Ok(cache) = registry.cache.read()
        && let Some(hit) = cache.get(&item.id)
    {
        return Ok(hit.clone());
    }
    let built = Arc::new(index::build_index(&item.root)?);
    if let Ok(mut cache) = registry.cache.write() {
        cache.insert(item.id.clone(), built.clone());
    }
    Ok(built)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::sync::Arc;

    use super::index_for;
    use crate::project;

    #[test]
    fn test_usage() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_path_buf();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("Cargo.toml"),
            "[package]\nname = \"demo\"\nversion = \"0.0.0\"\n",
        )
        .unwrap();
        fs::write(root.join("src/lib.rs"), "pub fn f() {}\n").unwrap();
        let registry = project::Registry::default();
        let item = project::ProjectRef {
            id: "demo".to_string(),
            name: "demo".to_string(),
            root,
        };
        let first = index_for(&registry, &item).unwrap();
        let second = index_for(&registry, &item).unwrap();
        assert!(Arc::ptr_eq(&first, &second));
        assert_eq!(first.crate_name, "demo");
    }
}
