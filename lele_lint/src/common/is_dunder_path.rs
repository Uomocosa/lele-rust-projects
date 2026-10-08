use std::path::Path;

use crate::Dunder;

pub(crate) fn is_dunder_path(rel_path: &Path, dunder: &Dunder) -> bool {
    let in_folder = rel_path.components().any(|c| {
        c.as_os_str()
            .to_str()
            .is_some_and(|name| dunder.folders.contains_key(name))
    });
    if in_folder {
        return true;
    }
    rel_path
        .file_stem()
        .and_then(|s| s.to_str())
        .is_some_and(|stem| dunder.files.iter().any(|file| file.as_str() == stem))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::path::Path;

    use super::is_dunder_path;
    use crate::Dunder;

    fn dunder_with(folder: &str, file: &str) -> Dunder {
        let mut folders = HashMap::new();
        folders.insert(folder.to_string(), "basic".to_string());
        Dunder {
            folders,
            files: vec![file.to_string()],
        }
    }

    #[test]
    fn test_usage() {
        let dunder = dunder_with("__basic__", "prelude");
        assert!(is_dunder_path(
            Path::new("player/__basic__/enums.rs"),
            &dunder
        ));
        assert!(is_dunder_path(Path::new("player/prelude.rs"), &dunder));
        assert!(!is_dunder_path(Path::new("player/player.rs"), &dunder));
    }
}
