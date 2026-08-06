// This is a Bevy system function (note: uses a dummy type to avoid an actual Bevy dep)
// It belongs in bevy_systems/ but lives in the domain root

pub struct Query;

pub fn tick_enemies(_q: Query) {
    tracing::debug!(target: "enemy", "ticking enemies");
}
