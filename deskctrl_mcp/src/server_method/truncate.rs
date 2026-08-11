pub fn truncate(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.trim_end_matches('\n').to_string()
    } else {
        let mut t: String = s.chars().take(max_chars).collect();
        t.push('\u{2026}');
        t
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {
        assert_eq!(super::truncate("short", 10), "short");
        assert_eq!(super::truncate("trailing\n", 10), "trailing");
        let long = super::truncate(&"x".repeat(50), 5);
        assert_eq!(long, "xxxxx\u{2026}");
    }
}
