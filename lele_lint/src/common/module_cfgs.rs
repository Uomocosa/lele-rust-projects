use crate::common;
use crate::ModuleInfoMap;

pub(crate) fn build(module_info: &ModuleInfoMap) -> common::ModuleCfgMap {
    let mut map = common::ModuleCfgMap::new();
    for info in module_info.values() {
        let base = common::module_path_of(&info.rel_path);
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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::build;
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
        assert!(cfg_map.contains_key(&vec!["discovery".to_string()]));
    }
}
