pub fn style_css() -> &'static str {
    include_str!("../../assets/style.css")
}

#[cfg(test)]
mod tests {
    use super::style_css;

    #[test]
    fn test_usage() {
        assert!(style_css().contains(".code"));
    }
}
