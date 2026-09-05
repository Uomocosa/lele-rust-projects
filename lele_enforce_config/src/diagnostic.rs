use crate::Severity;

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub crate_dir: String,
    pub code: String,
    pub severity: Severity,
    pub message: String,
    pub hint: String,
}

impl Diagnostic {
    pub fn new(
        crate_dir: impl Into<String>,
        code: impl Into<String>,
        severity: Severity,
        message: impl Into<String>,
        hint: impl Into<String>,
    ) -> Self {
        Self {
            crate_dir: crate_dir.into(),
            code: code.into(),
            severity,
            message: message.into(),
            hint: hint.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Diagnostic;
    use crate::Severity;

    #[test]
    fn test_usage() {
        let d = Diagnostic::new(
            "foo",
            "missing-task:lele:clippy",
            Severity::Error,
            "missing",
            "add it",
        );
        assert_eq!(d.crate_dir, "foo");
        assert_eq!(d.code, "missing-task:lele:clippy");
    }
}
