// This is a Bevy system function (note: uses dummy types to avoid actual Bevy dep)
// It belongs in bevy_systems/ but lives in the domain root

pub fn tick_enemies() {
    tracing::debug!(target: "enemy", "ticking enemies");
}
