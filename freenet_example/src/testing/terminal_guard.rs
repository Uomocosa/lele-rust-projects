use std::path::PathBuf;
use std::process::Child;

use super::terminal_guard_title;

pub struct TerminalGuard {
    pub child: Option<Child>,
    pub window_title: String,
    pub log: PathBuf,
}

impl Default for TerminalGuard {
    fn default() -> Self {
        Self {
            child: None,
            window_title: String::new(),
            log: PathBuf::new(),
        }
    }
}

#[rustfmt::skip]
impl TerminalGuard {
    #[must_use] pub fn title(&self) -> &str { terminal_guard_title::title(self) }
}

#[cfg(test)]
mod tests {
    use super::TerminalGuard;

    #[test]
    fn test_usage() {
        let _ = std::mem::size_of::<TerminalGuard>();
    }
}
