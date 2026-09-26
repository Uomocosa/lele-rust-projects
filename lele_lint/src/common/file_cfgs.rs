use std::path::Path;

use crate::common;

pub(crate) fn file_cfgs(map: &common::ModuleCfgMap, rel_path: &Path) -> Vec<String> {
    let module = common::module_path_of(rel_path);
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
