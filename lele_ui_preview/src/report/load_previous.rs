use std::path::Path;

use crate::report;

pub fn load_previous(out_dir: &Path) -> Option<report::Manifest> {
    let text = std::fs::read_to_string(out_dir.join(report::MANIFEST_FILE)).ok()?;
    serde_json::from_str(&text).ok()
}

#[cfg(test)]
mod tests {
    use super::load_previous;
    use crate::report;

    #[test]
    fn test_usage() {
        let dir = tempfile::tempdir().unwrap();
        assert!(load_previous(dir.path()).is_none());
        let manifest = report::Manifest {
            crate_name: "demo".to_string(),
            ..report::Manifest::default()
        };
        std::fs::write(
            dir.path().join(report::MANIFEST_FILE),
            serde_json::to_string(&manifest).unwrap(),
        )
        .unwrap();
        assert_eq!(load_previous(dir.path()).unwrap().crate_name, "demo");
    }
}
