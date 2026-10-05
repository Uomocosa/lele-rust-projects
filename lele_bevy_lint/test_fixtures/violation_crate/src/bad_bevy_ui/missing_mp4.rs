// Spawns UI and drives it over time, but only ships a png preview (E037).

pub struct Spawner;

pub fn setup(spawner: &mut Spawner) {
    spawner.spawn((Node,));
}

pub fn tick(time: Res<Time>) {
    let _ = time.delta_secs();
}

#[cfg(test)]
mod tests {
    use lele_bevy_preview::{run, scene::Scene};

    #[test]
    #[ignore = "headed recording"]
    fn missing_mp4_ui_png_preview() {
        let scene = Scene {
            name: String::from("missing_mp4"),
        };
        let _ = run(&scene, &Config::default(), "violation_crate");
    }
}
