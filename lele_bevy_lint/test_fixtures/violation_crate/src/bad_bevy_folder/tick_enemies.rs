// This is a Bevy system function (note: uses a dummy type to avoid an actual Bevy dep)
// It belongs in bevy_systems/ but lives in the domain root.
// Fixtures are parsed, never compiled, so the App below is illustrative only.

pub struct Query;

pub fn tick_enemies(_q: Query) {
    tracing::debug!(target: "enemy", "ticking enemies");
}

pub fn register_enemies() {
    let app = App;
    app.add_systems(Update, tick_enemies);
}
