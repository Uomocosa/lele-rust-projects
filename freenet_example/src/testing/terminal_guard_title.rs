use super::terminal_guard::TerminalGuard;

pub fn title(guard: &TerminalGuard) -> &str {
    &guard.window_title
}

#[cfg(test)]
mod tests {
    use super::title;
    use crate::testing::terminal_guard::TerminalGuard;

    #[test]
    fn test_usage() {
        let guard = TerminalGuard::default();
        assert_eq!(title(&guard), "");
    }
}
