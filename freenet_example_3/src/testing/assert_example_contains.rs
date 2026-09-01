use std::path::Path;

/// # Panics
/// Panics if the file does not exist or does not contain `needle`.
#[allow(clippy::panic)]
#[allow(clippy::unwrap_used)]
pub fn assert_example_contains(path: &str, needle: &str) {
    let p = Path::new(path);
    assert!(p.exists(), "example file {path} does not exist");
    let content = std::fs::read_to_string(p).unwrap_or_else(|_| panic!("failed to read {path}"));
    assert!(
        content.contains(needle),
        "example file {path} does not contain {needle:?}"
    );
}

#[cfg(test)]
mod tests {
    use super::assert_example_contains;

    #[test]
    fn test_usage() {
        assert_example_contains("examples/standalone_demo.rs", "TestNode");
    }
}
