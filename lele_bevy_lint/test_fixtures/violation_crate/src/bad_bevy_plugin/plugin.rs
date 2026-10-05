// A Plugin whose build spawns UI but ships no scene preview (E038).

pub struct Spawner;

pub fn spawn_ui(spawner: &mut Spawner) {
    spawner.spawn((Node,));
}

pub struct BadPlugin;

impl Plugin for BadPlugin {
    fn build(&self, app: &mut Spawner) {
        spawn_ui(app);
    }
}
