use regex::Regex;

pub fn strip_ansi(text: &str) -> String {
    let re = Regex::new(r"\x1b\[[0-9;]*m").expect("valid ansi regex");
    re.replace_all(text, "").into_owned()
}

#[cfg(test)]
mod tests {
    use super::strip_ansi;

    #[test]
    fn test_usage() {
        let s = "\u{1b}[2m2026\u{1b}[0m x=\u{1b}[3m42\u{1b}[0m";
        let out = strip_ansi(s);
        assert!(out.contains("x=42"));
        assert!(!out.contains('\u{1b}'));
    }
}
