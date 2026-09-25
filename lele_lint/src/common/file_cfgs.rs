use std::path::Path;

use super::module_cfgs::ModuleCfgMap;
use super::module_path_of;

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

// no test_usage necessary
