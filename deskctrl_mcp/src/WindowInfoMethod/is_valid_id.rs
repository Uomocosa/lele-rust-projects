/// True for an X11 window id of the form `0x` followed by 1..=16 hex digits.
///
/// Ids are passed straight to `import`/`xwd` as arguments (no shell), so this is not an
/// injection guard: it exists so that garbage fails with our own message instead of a
/// confusing ImageMagick stderr dump.
pub fn is_valid_id(id: &str) -> bool {
    let Some(digits) = id.strip_prefix("0x").or_else(|| id.strip_prefix("0X")) else {
        return false;
    };
    !digits.is_empty() && digits.len() <= 16 && digits.chars().all(|c| c.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::is_valid_id;

    #[test]
    fn test_usage() {
        assert!(is_valid_id("0x03a00004"));
        assert!(is_valid_id("0X1"));
        assert!(!is_valid_id("0x"));
        assert!(!is_valid_id("03a00004"));
        assert!(!is_valid_id("0xzz"));
        assert!(!is_valid_id("0x1; rm -rf /"));
        assert!(!is_valid_id("0x00000000000000001"));
    }
}
