pub fn slug(text: &str) -> String {
    let mut out = String::new();
    for c in text.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if !out.is_empty() && !out.ends_with('-') {
            out.push('-');
        }
    }
    let trimmed: String = out.trim_end_matches('-').chars().take(60).collect();
    let trimmed = trimmed.trim_end_matches('-').to_string();
    if trimmed.is_empty() {
        "root".to_string()
    } else {
        trimmed
    }
}

#[cfg(test)]
mod tests {
    use super::slug;

    #[test]
    fn test_usage() {
        assert_eq!(slug("/p/{id}/file/{*path}"), "p-id-file-path");
        assert_eq!(slug("/"), "root");
        assert_eq!(slug("Open 'Search'!"), "open-search");
    }
}
