use std::collections::HashMap;
use std::path::PathBuf;

use crate::module_info::ModuleInfo;

pub type ModuleInfoMap = HashMap<PathBuf, ModuleInfo>;

#[cfg(test)]
mod tests {
    use super::ModuleInfoMap;

    #[test]
    fn test_usage() {
        let map = ModuleInfoMap::new();
        assert!(map.is_empty());
    }
}
