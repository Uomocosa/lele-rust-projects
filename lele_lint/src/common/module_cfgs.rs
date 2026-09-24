use std::collections::HashMap;
use std::path::Path;

use crate::ModuleInfoMap;

pub(crate) type ModuleCfgMap = HashMap<Vec<String>, String>;

pub(crate) fn build(module_info: &ModuleInfoMap) -> ModuleCfgMap {
    let mut map = ModuleCfgMap::new();
    for info in module_info.values() {
        let base = module_path_of(&info.rel_path);
        for decl in &info.declarations {
            let Some(cfg) = &decl.cfg else {
                continue;
            };
            let mut child = base.clone();
            child.push(decl.name.clone());
            map.insert(child, cfg.clone());
        }
    }
    map
}

pub(crate) fn file_cfgs(map: &ModuleCfgMap, rel_path: &Path) -> Vec<String> {
    let module = module_path_of(rel_path);
    let mut cfgs = Vec::new();
    let mut prefix: Vec<String> = Vec::new();
    for segment in &module {
        prefix.push(segment.clone());
        if let Some(cfg) = map.get(&prefix) {
            cfgs.push(cfg.clone());
        }
    }
    cfgs
}

// needed helper: module path a source file contributes to
fn module_path_of(rel_path: &Path) -> Vec<String> {
    let mut components: Vec<String> = rel_path
        .components()
        .filter_map(|c| c.as_os_str().to_str().map(str::to_string))
        .collect();
    let Some(file_name) = components.pop() else {
        return Vec::new();
    };
    if is_index_file(&file_name) {
        return components;
    }
    let stem = file_name.strip_suffix(".rs").unwrap_or(&file_name);
    components.push(stem.to_string());
    components
}

// needed helper: crate root or directory index files map to their parent module
fn is_index_file(file_name: &str) -> bool {
    matches!(file_name, "mod.rs" | "lib.rs" | "main.rs")
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{build, file_cfgs};
    use crate::ModDecl;
    use crate::ModuleInfo;
    use crate::ModuleInfoMap;

    fn map_with(rel_path: &str, decl_name: &str, cfg: Option<&str>) -> ModuleInfoMap {
        let mut map = ModuleInfoMap::new();
        map.insert(
            PathBuf::from(rel_path),
            ModuleInfo {
                rel_path: PathBuf::from(rel_path),
                declarations: vec![ModDecl {
                    name: decl_name.to_string(),
                    is_public: true,
                    cfg: cfg.map(str::to_string),
                }],
                reexports: Vec::new(),
            },
        );
        map
    }

    #[test]
    fn test_usage() {
        let map = map_with("lib.rs", "discovery", Some("feature = \"room_lobby\""));
        let cfg_map = build(&map);
        assert_eq!(
            file_cfgs(&cfg_map, &PathBuf::from("discovery/client.rs")),
            vec!["feature = \"room_lobby\"".to_string()]
        );
        assert!(file_cfgs(&cfg_map, &PathBuf::from("p2p/mod.rs")).is_empty());
    }

    #[test]
    fn test_usage_nested_accumulates() {
        let mut map = map_with("lib.rs", "a", Some("feature = \"x\""));
        map.insert(
            PathBuf::from("a/mod.rs"),
            ModuleInfo {
                rel_path: PathBuf::from("a/mod.rs"),
                declarations: vec![ModDecl {
                    name: "b".to_string(),
                    is_public: true,
                    cfg: Some("feature = \"y\"".to_string()),
                }],
                reexports: Vec::new(),
            },
        );
        let cfg_map = build(&map);
        assert_eq!(
            file_cfgs(&cfg_map, &PathBuf::from("a/b/c.rs")),
            vec!["feature = \"x\"".to_string(), "feature = \"y\"".to_string()]
        );
    }
}
