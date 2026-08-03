#[derive(Debug)]
pub struct Diagnostic {
    pub file: std::path::PathBuf,
    pub line: usize,
    pub col: usize,
    pub code: String,
    pub message: String,
    pub severity: crate::severity::Severity,
}
