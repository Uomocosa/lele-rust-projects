// needed helper: parsing utilities

use super::snake_case_files::SnakeCaseFiles;
use crate::Diagnostic;
use crate::EntryKind;
use crate::Project;
use crate::Severity;

pub(crate) fn check(_self: &SnakeCaseFiles, project: &Project) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    for entry in &project.entries {
        let name = extract_name(&entry.relative_path, entry.kind == EntryKind::Directory);
        if let Some(name) = name {
            if !is_snake_case(name) {
                let kind = if entry.kind == EntryKind::Directory {
                    "directory"
                } else {
                    "filename"
                };
                diags.push(Diagnostic {
                    file: entry.absolute_path.clone(),
                    line: 1,
                    col: 0,
                    code: "E002".to_string(),
                    message: format!(
                        "{kind} `{name}` is not snake_case — rename it to use lowercase letters, digits, and underscores",
                        kind = kind,
                        name = name
                    ),
                    severity: Severity::Error,
                });
            }
        }
    }

    diags
}

fn extract_name(path: &std::path::Path, is_dir: bool) -> Option<&str> {
    if is_dir {
        path.file_name().and_then(|n| n.to_str())
    } else {
        path.file_stem().and_then(|n| n.to_str())
    }
}

fn is_snake_case(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    name.chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        && !name.starts_with('_')
        && !name.ends_with('_')
        && !name.contains("__")
}

#[cfg(test)]
mod tests {
    use super::is_snake_case;

    #[test]
    fn test_usage() {
        assert!(is_snake_case("snake_case"));
        assert!(is_snake_case("player"));
        assert!(is_snake_case("player_new"));
        assert!(!is_snake_case("PascalCase"));
        assert!(!is_snake_case("camelCase"));
        assert!(!is_snake_case("__double_underscore"));
        assert!(!is_snake_case("_leading"));
        assert!(!is_snake_case("trailing_"));
    }
}
