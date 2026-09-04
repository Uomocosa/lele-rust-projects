#[must_use]
pub fn random_letter() -> u8 {
    let mut buf = [0u8; 1];
    getrandom::getrandom(&mut buf).unwrap_or_default();
    let offset = buf[0] % 26;
    b'a'.checked_add(offset).unwrap_or(b'a')
}

#[cfg(test)]
mod tests {
    use super::random_letter;

    #[test]
    fn test_usage() {
        let l = random_letter();
        assert!(l.is_ascii_lowercase());
    }
}
