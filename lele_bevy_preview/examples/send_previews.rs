use std::path::{Path, PathBuf};

use lele_bevy_preview::deliver;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = std::env::args_os().nth(1).map_or_else(
        || std::env::temp_dir().join("lele-bevy-preview"),
        PathBuf::from,
    );
    let Some(manifest) = deliver::load_previous::load_previous(&dir) else {
        eprintln!("no manifest in {}", dir.display());
        return Ok(());
    };
    let mut sent = 0usize;
    for artifact in manifest.artifacts.values() {
        let path = artifact_path(&dir, artifact);
        if path.exists() && deliver::send_one::send_one(&path, &artifact.label)? {
            sent = sent.saturating_add(1);
        }
    }
    println!("sent {sent} artifact(s) from {}", dir.display());
    Ok(())
}

// needed helper: manifest stores metadata, not paths; rebuild the name `run` writes
fn artifact_path(dir: &Path, artifact: &deliver::Artifact) -> PathBuf {
    let ext = match artifact.media {
        deliver::Media::Png => "png",
        deliver::Media::Mp4 => "mp4",
    };
    dir.join(format!(
        "{}__{}.{}",
        artifact.scene,
        deliver::slug(&artifact.label),
        ext
    ))
}
