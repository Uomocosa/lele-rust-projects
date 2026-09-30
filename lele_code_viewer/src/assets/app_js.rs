pub fn app_js() -> &'static str {
    include_str!("../../assets/app.js")
}

#[cfg(test)]
mod tests {
    use super::app_js;

    #[test]
    fn test_usage() {
        assert!(app_js().contains("highlightHash"));
        assert!(app_js().contains("data-href"));
        assert!(app_js().contains("cb-adj"));
        assert!(app_js().contains("has-active"));
        assert!(app_js().contains("createDocumentFragment"));
        assert!(app_js().contains("chipByExt"));
    }
}
