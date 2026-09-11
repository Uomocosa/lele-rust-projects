// Spawns a Bevy visual component and ships the required ignored ui_png
// test with a file-stem-named capture (E029 compliant).

pub struct Sprite;
pub struct Spawner;

pub fn spawn_target(spawner: &mut Spawner) {
    spawner.spawn((Sprite,));
}

#[cfg(test)]
mod tests {
    #[test]
    #[ignore = "headed window"]
    fn target_ui_png_preview() {
        let shot =
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target.png");
        assert!(shot.exists());
        println!("PREVIEW_ARTIFACT={}", shot.display());
    }

    #[test]
    #[ignore = "headed recording"]
    fn target_ui_mp4_preview() {
        drive_cursor("target");
        let clip =
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target.mp4");
        assert!(clip.exists());
        println!("PREVIEW_ARTIFACT={}", clip.display());
    }
}
