pub fn derive_signature(text: &str, start_line: usize) -> String {
    let mut sig = String::new();
    for line in text.lines().skip(start_line.saturating_sub(1)) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with("//") || trimmed.starts_with("#[") || trimmed.starts_with("#![") {
            continue;
        }
        sig.push_str(trimmed);
        sig.push(' ');
        if line.contains('{') || line.contains(';') {
            break;
        }
        if sig.len() > 400 {
            break;
        }
    }
    if let Some(pos) = sig.find('{') {
        sig.truncate(pos);
    } else if let Some(pos) = sig.find(';') {
        sig.truncate(pos.saturating_add(1));
    }
    sig.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::derive_signature;

    #[test]
    fn test_usage() {
        let text = "/// doc\npub fn foo(a: u8) -> u8 {\n    a\n}\n";
        assert_eq!(derive_signature(text, 2), "pub fn foo(a: u8) -> u8");
        let text2 = "pub struct Bar {\n    x: u8,\n}\n";
        assert_eq!(derive_signature(text2, 1), "pub struct Bar");
        let text3 = "pub type Alias = u8;\n";
        assert_eq!(derive_signature(text3, 1), "pub type Alias = u8;");
    }
}
