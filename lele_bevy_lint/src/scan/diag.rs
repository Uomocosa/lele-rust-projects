use std::path::Path;

use lele_lint::Diagnostic;
use lele_lint::Origin;
use lele_lint::Project;

use crate::scan;

#[must_use]
pub fn diag(
    project: &Project,
    origin: Origin,
    rel_path: &Path,
    line: usize,
    code: &str,
    message: String,
) -> Diagnostic {
    Diagnostic {
        file: scan::file_path(project, origin, rel_path),
        line,
        col: 0,
        code: code.to_string(),
        message,
    }
}

#[cfg(test)]
mod tests {
    use super::diag;
    use lele_lint::Origin;
    use lele_lint::Project;
    use std::path::Path;

    #[test]
    fn test_usage() {
        let diagnostic = diag(
            &Project::default(),
            Origin::Src,
            Path::new("a.rs"),
            7,
            "E029",
            String::from("m"),
        );
        assert_eq!(diagnostic.code, "E029");
        assert_eq!(diagnostic.line, 7);
    }
}
