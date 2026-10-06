use crate::deliver;
use std::collections::BTreeMap;
use std::path::PathBuf;

// needed helper: fixture shared by the tests in this file
fn artifact(scene: &str, label: &str, fingerprint: &str, pixels: &str) -> deliver::Artifact {
    deliver::Artifact {
        scene: scene.to_string(),
        label: label.to_string(),
        fingerprint: fingerprint.to_string(),
        pixel_hash: pixels.to_string(),
        media: deliver::Media::Png,
        kind: crate::scene::Kind::System,
    }
}

// needed helper: fixture shared by the tests in this file
fn capture(label: &str, status: deliver::Status) -> deliver::Capture {
    deliver::Capture {
        path: PathBuf::from(format!("out/{label}.png")),
        artifact: artifact("ui", label, &format!("ui|{label}"), "hash"),
        status,
    }
}

// needed helper: fixture shared by the tests in this file
fn manifest(entries: Vec<deliver::Artifact>) -> deliver::Manifest {
    deliver::Manifest {
        crate_name: String::from("demo"),
        artifacts: entries
            .into_iter()
            .map(|entry| (entry.fingerprint.clone(), entry))
            .collect::<BTreeMap<_, _>>(),
    }
}

#[test]
// needed helper: fixture shared by the tests in this file
fn test_usage() {
    assert_eq!(deliver::slug("hover then press"), "hover_then_press");
    assert_eq!(deliver::slug("A-B.c"), "a_b_c");
    assert_eq!(deliver::fingerprint_of("ui", "rest"), "ui|rest");
    assert_eq!(deliver::hash_bytes(b"abc"), deliver::hash_bytes(b"abc"));
    assert_ne!(deliver::hash_bytes(b"abc"), deliver::hash_bytes(b"abd"));
    assert_eq!(
        deliver::hash_bytes(b"abc").len(),
        deliver::basic::constants::HASH_CHARS
    );
    assert!(deliver::sendable(deliver::Status::Changed));
    assert!(deliver::sendable(deliver::Status::New));
    assert!(deliver::sendable(deliver::Status::FirstRun));
    assert!(!deliver::sendable(deliver::Status::Same));
}

#[test]
// needed helper: fixture shared by the tests in this file
fn test_usage_status_distinguishes_first_run_new_changed_and_same() {
    let before = manifest(vec![artifact("ui", "rest", "a", "1")]);
    let same = artifact("ui", "rest", "a", "1");
    let changed = artifact("ui", "rest", "a", "2");
    let fresh = artifact("ui", "other", "b", "1");
    assert_eq!(
        deliver::state_status(None, &same),
        deliver::Status::FirstRun
    );
    assert_eq!(
        deliver::state_status(Some(&before), &same),
        deliver::Status::Same
    );
    assert_eq!(
        deliver::state_status(Some(&before), &changed),
        deliver::Status::Changed
    );
    assert_eq!(
        deliver::state_status(Some(&before), &fresh),
        deliver::Status::New
    );
}

#[test]
// needed helper: fixture shared by the tests in this file
fn test_usage_status_matches_by_scene_and_label_when_fingerprint_moved() {
    let before = manifest(vec![artifact("ui", "rest", "a", "1")]);
    let moved = artifact("ui", "rest", "a-renamed", "1");
    assert_eq!(
        deliver::state_status(Some(&before), &moved),
        deliver::Status::Same
    );
}

#[test]
// needed helper: fixture shared by the tests in this file
fn test_usage_collect_labels_every_artifact_with_a_status() {
    let rendered = vec![(
        PathBuf::from("out/rest.png"),
        String::from("rest"),
        String::from("h1"),
    )];
    let captures = deliver::collect("ui", crate::scene::Kind::System, &rendered, None).unwrap();
    assert_eq!(captures.len(), 1);
    assert_eq!(captures[0].status, deliver::Status::FirstRun);
    assert_eq!(captures[0].artifact.fingerprint, "ui|rest");
    let previous = manifest(vec![captures[0].artifact.clone()]);
    let again =
        deliver::collect("ui", crate::scene::Kind::System, &rendered, Some(&previous)).unwrap();
    assert_eq!(again[0].status, deliver::Status::Same);
}

#[test]
// needed helper: fixture shared by the tests in this file
fn test_usage_hash_file_reads_bytes_and_reports_missing_files() {
    let dir = tempfile::tempdir().unwrap();
    let png = dir.path().join("bytes.png");
    std::fs::write(&png, b"pixel-bytes").unwrap();
    assert_eq!(
        deliver::hash_file(&png).unwrap(),
        deliver::hash_bytes(b"pixel-bytes")
    );
    assert!(deliver::hash_file(&dir.path().join("absent.png")).is_err());
}

#[test]
// needed helper: fixture shared by the tests in this file
fn test_usage_manifest_round_trips_through_disk() {
    let dir = tempfile::tempdir().unwrap();
    assert!(deliver::load_previous(dir.path()).is_none());
    deliver::write_all(
        dir.path(),
        "demo",
        &[capture("rest", deliver::Status::FirstRun)],
    )
    .unwrap();
    let loaded = deliver::load_previous(dir.path()).unwrap();
    assert_eq!(loaded.crate_name, "demo");
    assert_eq!(loaded.artifacts.len(), 1);
    assert!(
        dir.path()
            .join(deliver::basic::constants::MANIFEST_FILE)
            .exists()
    );
    assert!(
        dir.path()
            .join(deliver::basic::constants::CHANGES_FILE)
            .exists()
    );
}

#[test]
// needed helper: fixture shared by the tests in this file
fn test_usage_update_manifest_merges_rather_than_replaces() {
    let dir = tempfile::tempdir().unwrap();
    deliver::write_all(
        dir.path(),
        "demo",
        &[capture("rest", deliver::Status::FirstRun)],
    )
    .unwrap();
    deliver::update_manifest(
        dir.path(),
        "demo",
        &[capture("hover", deliver::Status::New)],
        None,
    )
    .unwrap();
    let loaded = deliver::load_previous(dir.path()).unwrap();
    assert_eq!(loaded.artifacts.len(), 2);
    assert!(loaded.artifacts.contains_key("ui|rest"));
    assert!(loaded.artifacts.contains_key("ui|hover"));
}

#[test]
// needed helper: fixture shared by the tests in this file
fn test_usage_changes_markdown_lists_only_what_moved() {
    let all = vec![
        capture("rest", deliver::Status::Same),
        capture("hover", deliver::Status::Changed),
        capture("press", deliver::Status::New),
    ];
    let text = deliver::changes_markdown(&all);
    assert!(text.contains("1 changed, 1 new, 1 unchanged"));
    assert!(text.contains("hover"));
    let quiet = deliver::changes_markdown(&[capture("rest", deliver::Status::Same)]);
    assert!(quiet.contains("No UI state changed"));
    assert!(deliver::changes_markdown(&[]).contains("No UI state changed"));
    let first = deliver::changes_markdown(&[capture("rest", deliver::Status::FirstRun)]);
    assert!(first.contains("0 changed, 1 new"));
}
// no test_usage necessary
