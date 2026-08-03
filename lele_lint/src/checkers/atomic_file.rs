use std::path::Path;

use crate::checker::Checker;
use crate::config::Config;
use crate::diagnostic::Diagnostic;
use crate::project::Project;
use crate::severity::Severity;

use super::atomic_file_register;
// needed helper: parsing utilities

pub struct AtomicFile;

impl Checker for AtomicFile {
    fn name(&self) -> &'static str {
        "atomic_file"
    }

    fn code(&self) -> &'static str {
        "E001"
    }

    fn check(&self, project: &Project) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (rel_path, file) in &project.parsed_files {
            let file_name = rel_path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            if is_exempt_path(rel_path, file_name) {
                continue;
            }

            let pub_items = collect_pub_items(file);
            if pub_items.is_empty() {
                continue;
            }

            let file_stem = rel_path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            let primary = &pub_items[0];

            check_filename_match(&primary.name, file_stem, rel_path, project, &mut diags);

            for extra in &pub_items[1..] {
                let suggested_file = format!("{}_{}.rs", file_stem, extra.name);
                diags.push(Diagnostic {
                    file: project.src_dir.join(rel_path),
                    line: 1,
                    col: 0,
                    code: "E001".to_string(),
                    message: format!(
                        "only one public item per file — move `pub {} {}` to `{}`",
                        extra.kind.kind_str(),
                        extra.name,
                        suggested_file
                    ),
                    severity: Severity::Error,
                });
            }
        }

        diags
    }
}

#[rustfmt::skip]
impl AtomicFile {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
        atomic_file_register::register(checkers, config)
    }
}

struct PubItem {
    name: String,
    kind: PubItemKind,
}

enum PubItemKind {
    Struct,
    Enum,
    Fn,
}

impl PubItemKind {
    fn kind_str(&self) -> &'static str {
        match self {
            PubItemKind::Struct => "struct",
            PubItemKind::Enum => "enum",
            PubItemKind::Fn => "fn",
        }
    }
}

fn is_exempt_path(rel_path: &Path, file_name: &str) -> bool {
    if file_name == "mod.rs" || file_name == "lib.rs" || file_name == "constants.rs" {
        return true;
    }
    rel_path
        .components()
        .any(|c| c.as_os_str().to_str() == Some("tests"))
}

fn collect_pub_items(file: &syn::File) -> Vec<PubItem> {
    let mut items = Vec::new();
    for item in &file.items {
        match item {
            syn::Item::Struct(s) if matches!(s.vis, syn::Visibility::Public(_)) => {
                items.push(PubItem {
                    name: s.ident.to_string(),
                    kind: PubItemKind::Struct,
                });
            }
            syn::Item::Enum(e) if matches!(e.vis, syn::Visibility::Public(_)) => {
                items.push(PubItem {
                    name: e.ident.to_string(),
                    kind: PubItemKind::Enum,
                });
            }
            syn::Item::Fn(f) if matches!(f.vis, syn::Visibility::Public(_)) => {
                items.push(PubItem {
                    name: f.sig.ident.to_string(),
                    kind: PubItemKind::Fn,
                });
            }
            _ => {}
        }
    }
    items
}

fn check_filename_match(
    name: &str,
    file_stem: &str,
    rel_path: &Path,
    project: &Project,
    diags: &mut Vec<Diagnostic>,
) {
    let expected = to_snake_case(name);

    if expected == file_stem {
        return;
    }

    if file_stem.contains('_') {
        if let Some((_prefix, suffix)) = file_stem.rsplit_once('_') {
            if expected.ends_with(suffix) {
                return;
            }
        }
    }

    diags.push(Diagnostic {
        file: project.src_dir.join(rel_path),
        line: 1,
        col: 0,
        code: "E001".to_string(),
        message: format!("filename mismatch — `{file_stem}.rs` should be `{expected}.rs`"),
        severity: Severity::Error,
    });
}

fn to_snake_case(pascal: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = pascal.chars().collect();
    let len = chars.len();

    for (i, &c) in chars.iter().enumerate() {
        if c.is_uppercase() {
            let preceded_by_lower = i > 0 && chars[i - 1].is_lowercase();
            let followed_by_lower = i + 1 < len && chars[i + 1].is_lowercase();
            let preceded_by_upper = i > 0 && chars[i - 1].is_uppercase();

            if preceded_by_lower || (followed_by_lower && preceded_by_upper) {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::to_snake_case;

    #[test]
    fn test_usage() {
        assert_eq!(to_snake_case("ConstructorNoSkip"), "constructor_no_skip");
        assert_eq!(to_snake_case("DomainImport"), "domain_import");
        assert_eq!(to_snake_case("SnakeCaseFiles"), "snake_case_files");
        assert_eq!(to_snake_case("NoPositional"), "no_positional");
        assert_eq!(to_snake_case("Player"), "player");
        assert_eq!(to_snake_case("PlayerEvent"), "player_event");
    }
}
