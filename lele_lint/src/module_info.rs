// needed helper: syn parsing utilities for mod.rs declarations
use std::path::PathBuf;

use atomic_delegate_macros::atomic_delegates;

use crate::Entry;
use crate::ModDecl;
use crate::ModuleInfoMap;
use crate::Reexport;

#[derive(Debug, Clone)]
pub struct ModuleInfo {
    pub rel_path: PathBuf,
    pub declarations: Vec<ModDecl>,
    pub reexports: Vec<Reexport>,
}

#[atomic_delegates]
impl ModuleInfo {
    pub fn build(entries: &[Entry]) -> ModuleInfoMap {}
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::Entry;
    use crate::EntryKind;
    use crate::ModuleInfo;

    #[test]
    fn test_usage() {
        let dir = tempfile::tempdir().unwrap();
        let rel = PathBuf::from("player/mod.rs");
        let abs = dir.path().join("mod.rs");
        std::fs::write(
            &abs,
            "mod player;\nmod player_new;\npub mod bevy_systems;\npub use player::Player;\n",
        )
        .unwrap();
        let entries = vec![Entry {
            relative_path: rel.clone(),
            absolute_path: abs,
            kind: EntryKind::File,
        }];
        let map = ModuleInfo::build(&entries);
        let info = map.get(&rel).unwrap();
        assert_eq!(info.declarations.len(), 3);
        assert_eq!(info.declarations[0].name, "player");
        assert!(!info.declarations[0].is_public);
        assert_eq!(info.declarations[2].name, "bevy_systems");
        assert!(info.declarations[2].is_public);
        assert_eq!(info.reexports.len(), 1);
        assert_eq!(info.reexports[0].segments, vec!["player", "Player"]);
    }
}
