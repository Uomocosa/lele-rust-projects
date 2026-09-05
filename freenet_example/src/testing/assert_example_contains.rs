use std::path::Path;

#[must_use]
pub fn assert_example_contains(path: &str, needle: &str) -> bool {
    let p = Path::new(path);
    if !p.exists() {
        eprintln!("example file {path} does not exist");
        return false;
    }
    let content = std::fs::read_to_string(p).unwrap_or_default();
    if content.contains(needle) {
        true
    } else {
        eprintln!("example file {path} does not contain {needle:?}");
        false
    }
}

#[cfg(test)]
mod tests {
    use super::assert_example_contains;

    #[test]
    fn test_usage() {
        assert!(assert_example_contains(
            "examples/standalone_demo.rs",
            "TestNode"
        ));
    }
}
