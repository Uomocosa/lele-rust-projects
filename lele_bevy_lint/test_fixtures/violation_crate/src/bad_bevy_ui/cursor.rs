// Spawns a Bevy visual component but defines no ui_png test (E029).
// Uses dummy types to avoid an actual Bevy dependency; the checker only
// matches on the last path segment. The receiver is deliberately not named
// Commands so the bevy_folder (E008) checker stays quiet here.

pub struct Sprite;
pub struct Spawner;

pub fn setup(spawner: &mut Spawner) {
    spawn_cursor(spawner);
}

pub fn spawn_cursor(spawner: &mut Spawner) {
    spawner.spawn((Sprite,));
}
