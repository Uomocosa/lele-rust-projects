use crate::{OutputBuffer, output_buffer::MAX_BYTES};

/// Append output, trimming whole lines off the front to stay under `MAX_BYTES`.
pub fn push(buf: &mut OutputBuffer, line: &str) {
    buf.text.push_str(line);

    while buf.text.len() > MAX_BYTES {
        // Trim whole lines so the retained text never starts mid-line.
        let cut = match buf.text.find('\n') {
            Some(i) => i + 1,
            None => buf.text.len(),
        };
        buf.text.drain(..cut);
        buf.dropped += cut;
    }
}

#[cfg(test)]
mod tests {
    use super::push;
    use crate::{OutputBuffer, output_buffer::MAX_BYTES};

    #[test]
    fn test_usage() {
        let mut buf = OutputBuffer::default();
        push(&mut buf, "one\n");
        push(&mut buf, "two\n");
        assert_eq!(buf.text, "one\ntwo\n");
        assert_eq!(buf.dropped, 0);

        let big = "x".repeat(MAX_BYTES);
        push(&mut buf, &format!("{big}\n"));
        assert!(buf.text.len() <= MAX_BYTES);
        assert!(buf.dropped > 0);
        // absolute end still counts everything ever written
        assert_eq!(buf.end(), 8 + MAX_BYTES + 1);
        // the retained text starts at a line boundary
        assert!(!buf.text.starts_with('\n'));
    }
}
