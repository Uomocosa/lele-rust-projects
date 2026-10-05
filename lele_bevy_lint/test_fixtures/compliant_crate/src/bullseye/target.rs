// Spawns a UI visual and ships the required ignored ui_png preview routed
// through the harness (E029 + E039 compliant).

#[derive(Component)]
pub struct Target;

pub struct Spawner;

pub fn spawn_target(spawner: &mut Spawner) {
    spawner.spawn((Target, Sprite));
}

#[cfg(test)]
mod tests {
    use lele_bevy_preview::{run, scene::Scene};

    #[test]
    #[ignore = "headed preview"]
    fn target_ui_png_preview() {
        let _ = Target;
        let scene = Scene {
            name: String::from("target"),
        };
        let _ = run(&scene, &Config::default(), "compliant_crate");
    }
}
