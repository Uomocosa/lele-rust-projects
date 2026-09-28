use std::path::Path;

use crate::Error;
use crate::report;

pub fn write_report(
    out_dir: &Path,
    manifest: &report::Manifest,
    previous: Option<&report::Manifest>,
) -> Result<(), Error> {
    std::fs::write(
        out_dir.join(report::MANIFEST_FILE),
        serde_json::to_string_pretty(manifest)?,
    )?;
    let mut atlases = Vec::new();
    for (group, screen) in screens(manifest) {
        let pngs: Vec<String> = manifest
            .states
            .iter()
            .filter(|s| s.group == group && s.screen == screen)
            .map(|s| s.png.clone())
            .collect();
        let atlas = report::write_atlas(out_dir, &group, &screen, &pngs)?;
        atlases.push((group, screen, atlas));
    }
    std::fs::write(
        out_dir.join(report::CHANGES_FILE),
        report::changes_markdown(previous, manifest),
    )?;
    std::fs::write(
        out_dir.join(report::INDEX_FILE),
        report::index_html(manifest, previous, &atlases),
    )?;
    Ok(())
}

// needed helper: (group, screen) pairs in first-seen order
fn screens(manifest: &report::Manifest) -> Vec<(String, String)> {
    let mut seen: Vec<(String, String)> = Vec::new();
    for state in &manifest.states {
        let key = (state.group.clone(), state.screen.clone());
        if !seen.contains(&key) {
            seen.push(key);
        }
    }
    seen
}

#[cfg(test)]
mod tests {
    use image::{Rgba, RgbaImage};

    use super::write_report;
    use crate::report;

    #[test]
    fn test_usage() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("mobile/root")).unwrap();
        RgbaImage::from_pixel(40, 80, Rgba([0, 0, 0, 255]))
            .save(dir.path().join("mobile/root/s000__start.png"))
            .unwrap();
        let manifest = report::Manifest {
            crate_name: "demo".to_string(),
            driver: "web".to_string(),
            states: vec![report::StateRecord {
                id: "s000".to_string(),
                group: "mobile".to_string(),
                screen: "/".to_string(),
                path: Vec::new(),
                location: "/".to_string(),
                png: "mobile/root/s000__start.png".to_string(),
                fingerprint: "f".to_string(),
                pixel_hash: "p".to_string(),
            }],
            ..report::Manifest::default()
        };
        write_report(dir.path(), &manifest, None).unwrap();
        for file in [
            "manifest.json",
            "CHANGES.md",
            "index.html",
            "mobile/root/atlas.png",
        ] {
            assert!(dir.path().join(file).exists(), "{file}");
        }
    }
}
