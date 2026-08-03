pub fn poll_input(key_code: u32, buffer: &mut Vec<u32>) {
    buffer.push(key_code);
    tracing::debug!(target: "player", key_code = key_code);
}

#[cfg(test)]
mod tests {
    use super::poll_input;

    #[test]
    fn test_usage() {
        let mut buf = Vec::new();
        poll_input(42, &mut buf);
        assert_eq!(buf, vec![42]);
    }
}
