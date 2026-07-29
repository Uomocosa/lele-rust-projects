pub fn help_text() -> String {
    String::from(
        "Commands:\n\
         increment, inc, +  - Increment the counter\n\
         status, s          - Show current count\n\
         quit, q, exit      - Quit the application\n\
         help, h            - Show this help",
    )
}

#[cfg(test)]
mod tests {
    use super::help_text;

    #[test]
    fn test_usage() {
        let text = help_text();
        assert!(text.contains("increment"));
        assert!(text.contains("status"));
        assert!(text.contains("quit"));
        assert!(text.contains("help"));
    }
}
