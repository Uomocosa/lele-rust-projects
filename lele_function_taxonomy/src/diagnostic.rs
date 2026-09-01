#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub file: std::path::PathBuf,
    pub line: usize,
    pub col: usize,
    pub code: String,
    pub message: String,
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}:{}: {} {}",
            self.file.display(),
            self.line,
            self.col,
            self.code,
            self.message
        )
    }
}

// no test_usage necessary
