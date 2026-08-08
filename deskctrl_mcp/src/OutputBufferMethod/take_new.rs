use crate::OutputBuffer;

/// Return everything appended since the last call and advance the cursor.
pub fn take_new(buf: &mut OutputBuffer) -> String {
    let start = buf.cursor.saturating_sub(buf.dropped).min(buf.text.len());
    let out = buf.text[start..].to_string();
    buf.cursor = buf.end();
    out
}

#[cfg(test)]
mod tests {
    use super::take_new;
    use crate::OutputBuffer;

    #[test]
    fn test_usage() {
        let mut buf = OutputBuffer::default();
        buf.push("first\n");
        assert_eq!(take_new(&mut buf), "first\n");
        buf.push("second\n");
        assert_eq!(take_new(&mut buf), "second\n");
        assert_eq!(take_new(&mut buf), "");

        // A cursor left behind by a trim must not panic; the caller missed the trimmed output,
        // so it gets everything still retained.
        buf.cursor = 0;
        buf.dropped = 999;
        assert_eq!(take_new(&mut buf), "first\nsecond\n");
    }
}
