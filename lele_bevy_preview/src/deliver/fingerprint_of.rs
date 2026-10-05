#[must_use]
pub fn fingerprint_of(scene: &str, label: &str) -> String {
    format!("{scene}|{label}")
}
// no test_usage necessary
