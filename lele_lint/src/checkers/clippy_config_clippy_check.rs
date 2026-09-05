use super::clippy_config_clippy::ClippyConfigClippy;
use crate::Diagnostic;
use crate::Project;
use crate::Severity;

const REQUIRED_KEYS: &[&str] = &[
    "allow-unwrap-in-tests",
    "allow-expect-in-tests",
    "allow-panic-in-tests",
    "allow-indexing-slicing-in-tests",
];

pub(crate) fn check(_self: &ClippyConfigClippy, project: &Project) -> Vec<Diagnostic> {
    let path = project.root.join("clippy.toml");
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => {
            return vec![Diagnostic {
                file: path,
                line: 1,
                col: 0,
                code: ClippyConfigClippy::CODE.to_string(),
                message: "clippy.toml not found — add it with allow-unwrap-in-tests, allow-expect-in-tests, allow-panic-in-tests, allow-indexing-slicing-in-tests = true".to_string(),
                severity: Severity::Error,
            }];
        }
    };
    let value: toml::Value = match content.parse() {
        Ok(v) => v,
        Err(e) => {
            return vec![Diagnostic {
                file: path,
                line: 1,
                col: 0,
                code: ClippyConfigClippy::CODE.to_string(),
                message: format!("clippy.toml parse error: {e}"),
                severity: Severity::Error,
            }];
        }
    };
    let table = match value.as_table() {
        Some(t) => t,
        None => {
            return vec![Diagnostic {
                file: path,
                line: 1,
                col: 0,
                code: ClippyConfigClippy::CODE.to_string(),
                message: "clippy.toml must be a table with allow-* = true entries".to_string(),
                severity: Severity::Error,
            }];
        }
    };
    let mut diags = Vec::new();
    for key in REQUIRED_KEYS {
        match table.get(*key) {
            Some(toml::Value::Boolean(true)) => {}
            _ => diags.push(Diagnostic {
                file: path.clone(),
                line: 1,
                col: 0,
                code: ClippyConfigClippy::CODE.to_string(),
                message: format!(
                    "clippy.toml missing {key} = true — minimum clippy config requires it"
                ),
                severity: Severity::Error,
            }),
        }
    }
    diags
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {
        let content = r"
allow-unwrap-in-tests = true
allow-expect-in-tests = true
allow-panic-in-tests = true
allow-indexing-slicing-in-tests = true
";
        let value: toml::Value = content.parse().unwrap();
        assert!(value.get("allow-unwrap-in-tests").is_some());
    }
}
