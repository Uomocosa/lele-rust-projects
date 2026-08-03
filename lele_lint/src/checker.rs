// lele_lint: allow E001
use crate::project::Project;

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub file: std::path::PathBuf,
    pub line: usize,
    pub col: usize,
    pub code: String,
    pub message: String,
    pub severity: Severity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Severity {
    Error,
    Warning,
}

pub trait Checker {
    fn name(&self) -> &'static str;
    fn code(&self) -> &'static str;
    fn check(&self, project: &Project) -> Vec<Diagnostic>;
}
