use std::fmt::Write;

use lele_lint::Project;

use crate::scan;

pub fn report(project: &Project) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "{:<44} {:<26} {:<18} {:<16} {:<3} {:<3} {:<3}",
        "file", "markers", "visual", "anim driver", "png", "mp4", "scene"
    );
    for row in &rows(project) {
        let _ = writeln!(
            out,
            "{:<44} {:<26} {:<18} {:<16} {:<3} {:<3} {:<3}",
            row.file,
            row.markers,
            row.visual,
            row.anim,
            mark(row.png),
            mark(row.mp4),
            mark(row.scene)
        );
    }
    out
}

struct Row {
    file: String,
    markers: String,
    visual: String,
    anim: String,
    png: bool,
    mp4: bool,
    scene: bool,
}

// needed helper: one inventory row per file that has UI or previews
fn rows(project: &Project) -> Vec<Row> {
    let mut rows: Vec<Row> = project
        .parsed_files
        .iter()
        .filter_map(|(rel_path, file)| {
            let visuals = scan::prod_visuals(file);
            let previews = scan::collect_previews(file);
            let relevant_plugin =
                scan::defines_plugin(file) && scan::dir_spawns_ui(project, rel_path);
            if visuals.is_empty() && previews.is_empty() && !relevant_plugin {
                return None;
            }
            Some(Row {
                file: rel_path.display().to_string(),
                markers: join_unique(scan::declared_components(file)),
                visual: join_unique(visuals.iter().map(|found| found.visual.clone()).collect()),
                anim: join_unique(scan::drivers(file)),
                png: previews.iter().any(|p| p.kind == scan::PreviewKind::Png),
                mp4: previews.iter().any(|p| p.kind == scan::PreviewKind::Mp4),
                scene: previews.iter().any(|p| p.kind == scan::PreviewKind::Scene),
            })
        })
        .collect();
    rows.sort_by(|a, b| a.file.cmp(&b.file));
    rows
}

// needed helper: comma-joined sorted-unique idents
fn join_unique(mut items: Vec<String>) -> String {
    items.sort();
    items.dedup();
    items.join(",")
}

// needed helper: coverage x / - marker
fn mark(present: bool) -> &'static str {
    if present { "x" } else { "-" }
}

#[cfg(test)]
mod tests {
    use super::report;
    use lele_lint::Project;
    use std::path::PathBuf;

    #[test]
    fn test_usage() {
        let mut project = Project::default();
        project.parsed_files.insert(
            PathBuf::from("ui/root.rs"),
            syn::parse_str("pub fn setup(s: &mut S) { s.spawn((Node,)); }").unwrap(),
        );
        let text = report(&project);
        assert!(text.contains("ui/root.rs"));
        assert!(text.contains("Node"));
    }
}
