pub fn app_js() -> &'static str {
    include_str!("../../assets/app.js")
}

#[cfg(test)]
mod tests {
    use super::app_js;

    #[test]
    fn test_usage() {
        assert!(app_js().contains("highlightHash"));
    }
}
