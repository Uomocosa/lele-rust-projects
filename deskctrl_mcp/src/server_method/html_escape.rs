pub fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {
        assert_eq!(super::html_escape("<b>&</b>"), "&lt;b&gt;&amp;&lt;/b&gt;");
        assert_eq!(super::html_escape("plain"), "plain");
    }
}
