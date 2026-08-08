use crate::OutputBuffer;

/// First retained line containing `pattern` (plain substring, not a regex).
///
/// Scans the whole transcript, including output already returned by `read_output`.
pub fn find_line(buf: &OutputBuffer, pattern: &str) -> Option<String> {
    buf.text
        .lines()
        .find(|line| line.contains(pattern))
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::find_line;
    use crate::OutputBuffer;

    #[test]
    fn test_usage() {
        let mut buf = OutputBuffer::default();
        buf.push("[OUT] starting up\n[OUT] connected, running key=abc count=0\n");
        buf.take_new(); // consumed by read_output — must still be findable
        let line = find_line(&buf, "connected, running").unwrap();
        assert!(line.contains("key=abc"));
        assert_eq!(find_line(&buf, "missing"), None);
    }
}
