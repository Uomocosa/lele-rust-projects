// Recorder signals without a matching mp4 preview (E029 kind case).

pub struct Spawner;

pub fn setup(spawner: &mut Spawner) {
    let _ = spawner;
}

#[cfg(test)]
mod tests {
    #[test]
    #[ignore = "headed recording"]
    fn wheel_ui_png_preview() {
        drive_cursor("wheel");
        let shot =
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("missing_mp4.png");
        assert!(shot.exists());
        println!("PREVIEW_ARTIFACT={}", shot.display());
    }
}
