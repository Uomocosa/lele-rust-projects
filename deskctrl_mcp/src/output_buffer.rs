use crate::output_buffer_method;

/// Append-only transcript of a process's output, plus the `read_output` cursor.
///
/// Kept append-only so `wait_for_output` can scan output that `read_output` already returned —
/// a draining buffer loses the very line a caller is waiting for. Bounded from the front, since
/// a chatty process (a freenet node logs hundreds of lines a second) would otherwise grow
/// without limit; `dropped` keeps the cursor meaningful across trims.
#[derive(Debug, Default)]
pub struct OutputBuffer {
    /// The retained tail of the transcript.
    pub text: String,
    /// Bytes trimmed off the front, so positions stay absolute.
    pub dropped: usize,
    /// Absolute position `read_output` has consumed up to.
    pub cursor: usize,
}

/// Retain at most this much output per process.
pub const MAX_BYTES: usize = 4 * 1024 * 1024;

#[rustfmt::skip]
impl OutputBuffer {
    pub fn push(&mut self, line: &str) { output_buffer_method::push(self, line) }
    pub fn take_new(&mut self) -> String { output_buffer_method::take_new(self) }
    pub fn find_line(&self, pattern: &str) -> Option<String> { output_buffer_method::find_line(self, pattern) }
    pub fn end(&self) -> usize { self.dropped + self.text.len() }
}

#[cfg(test)]
mod tests {
    use crate::OutputBuffer;

    #[test]
    fn test_usage() {
        let mut buf = OutputBuffer::default();
        buf.push("hello\n");
        assert_eq!(buf.end(), 6);
        assert_eq!(buf.take_new(), "hello\n");
        assert_eq!(buf.take_new(), "");
        // already-read output is still findable
        assert_eq!(buf.find_line("hello"), Some("hello".to_string()));
        assert_eq!(buf.find_line("nope"), None);
    }
}
