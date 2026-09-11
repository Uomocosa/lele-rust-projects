// Reachable production spawn with a preview that breaks the contract:
// no ignore, no assert, no PREVIEW_ARTIFACT println, wrong stem literal.

pub struct Sprite;
pub struct Spawner;

pub fn setup(spawner: &mut Spawner) {
    spawn_badge(spawner);
}

pub fn spawn_badge(spawner: &mut Spawner) {
    spawner.spawn((Sprite,));
}

#[cfg(test)]
mod tests {
    #[test]
    fn badge_ui_png_preview() {
        let shot =
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("wrong.png");
        let _ = shot;
    }
}
