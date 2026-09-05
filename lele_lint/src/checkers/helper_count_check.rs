use std::path::Path;

use syn::spanned::Spanned;

use super::helper_count::HelperCount;
use crate::Diagnostic;
use crate::EntryKind;
use crate::Project;
use crate::Severity;

const MAX_PRIVATE_HELPERS: usize = 2;

const ANNOTATION: &str = "// needed helper:";

pub(crate) fn check(_self: &HelperCount, project: &Project) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    for (rel_path, file) in &project.parsed_files {
        let file_name = rel_path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        if file_name == "mod.rs" || file_name == "lib.rs" || file_name == "constants.rs" {
            continue;
        }

        if rel_path
            .components()
            .any(|c| c.as_os_str().to_str() == Some("tests"))
        {
            continue;
        }

        let Some(source) = read_source(project, rel_path) else {
            continue;
        };

        let private_count = count_unannotated_private_helpers(file, &source);

        if private_count > MAX_PRIVATE_HELPERS {
            diags.push(Diagnostic {
                file: project.src_dir.join(rel_path),
                line: 1,
                col: 0,
                code: "E015".to_string(),
                message: format!(
                    "{} unannotated helper functions (max {}). Annotate context-specific helpers with `{}` on the line above each function; extract reusable ones into atomic delegate files",
                    private_count,
                    MAX_PRIVATE_HELPERS,
                    ANNOTATION
                ),
                severity: Severity::Error,
            });
        }

        let pub_like_fns = collect_pub_like_top_level_fns(file);

        if pub_like_fns.len() > 1 {
            for func in &pub_like_fns {
                diags.push(Diagnostic {
                    file: project.src_dir.join(rel_path),
                    line: func.sig.fn_token.span().start().line,
                    col: 0,
                    code: "E015".to_string(),
                    message: format!(
                        "{} public/pub(crate) top-level functions in this file (fn `{}` among them); only the file's single core function may be pub/pub(crate) — extract the others into their own files",
                        pub_like_fns.len(),
                        func.sig.ident
                    ),
                    severity: Severity::Error,
                });
            }
        }
    }

    diags
}

fn read_source(project: &Project, rel_path: &Path) -> Option<String> {
    let entry = project
        .entries
        .iter()
        .find(|e| e.relative_path == rel_path && e.kind == EntryKind::File)?;

    std::fs::read_to_string(&entry.absolute_path).ok()
}

fn count_unannotated_private_helpers(file: &syn::File, source: &str) -> usize {
    let lines: Vec<&str> = source.lines().collect();

    file.items
        .iter()
        .filter_map(|item| match item {
            syn::Item::Fn(func) if matches!(func.vis, syn::Visibility::Inherited) => Some(func),
            _ => None,
        })
        .filter(|func| {
            // `line` is 1-based; step back over blank lines and attributes to reach
            // the nearest line that could carry the annotation.
            let line = func.sig.fn_token.span().start().line;
            let mut idx = line.saturating_sub(1);

            while idx > 0 {
                idx = idx.saturating_sub(1);
                let candidate = lines.get(idx).map(|l| l.trim()).unwrap_or("");
                if candidate.is_empty() || candidate.starts_with("#[") {
                    continue;
                }
                return !candidate.starts_with(ANNOTATION);
            }

            true
        })
        .count()
}

// needed helper: shared by check() and its tests to bucket pub/pub(crate) fns separately from private helpers
fn collect_pub_like_top_level_fns(file: &syn::File) -> Vec<&syn::ItemFn> {
    file.items
        .iter()
        .filter_map(|item| match item {
            syn::Item::Fn(func) if !matches!(func.vis, syn::Visibility::Inherited) => Some(func),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{collect_pub_like_top_level_fns, count_unannotated_private_helpers};

    fn parse(source: &str) -> (syn::File, String) {
        (syn::parse_str(source).unwrap(), source.to_string())
    }

    #[test]
    fn test_usage_counts_helpers() {
        let (file, source) = parse(
            "pub struct Foo;
pub fn bar() {}
fn helper_a() {}
fn helper_b() {}
fn helper_c() {}
#[cfg(test)] mod tests { fn test_usage() {} }",
        );
        assert_eq!(count_unannotated_private_helpers(&file, &source), 3);
    }

    #[test]
    fn test_usage_skips_impl_fns() {
        let (file, source) = parse(
            "pub struct Foo;
impl Foo { fn method_a() {} fn method_b() {} fn method_c() {} }
fn helper_a() {}",
        );
        assert_eq!(count_unannotated_private_helpers(&file, &source), 1);
    }

    #[test]
    fn test_usage_skips_pub_crate_fns() {
        let (file, source) = parse(
            "pub(crate) fn walk() {}
pub(super) fn parse_all() {}
fn helper_a() {}",
        );
        assert_eq!(count_unannotated_private_helpers(&file, &source), 1);
    }

    #[test]
    fn test_usage_annotation_excuses_one_fn() {
        let (file, source) = parse(
            "// needed helper: first
fn helper_a() {}
fn helper_b() {}
fn helper_c() {}",
        );
        assert_eq!(count_unannotated_private_helpers(&file, &source), 2);
    }

    #[test]
    fn test_usage_annotation_skips_attributes_and_blanks() {
        let (file, source) = parse(
            "// needed helper: still applies

#[allow(dead_code)]
fn helper_a() {}",
        );
        assert_eq!(count_unannotated_private_helpers(&file, &source), 0);
    }

    #[test]
    fn test_usage_file_level_annotation_does_not_excuse_all() {
        let (file, source) = parse(
            "// needed helper: parsing utilities

use std::path::Path;

fn helper_a() {}
fn helper_b() {}
fn helper_c() {}",
        );
        assert_eq!(count_unannotated_private_helpers(&file, &source), 3);
    }

    #[test]
    fn test_usage_two_pub_fns_flagged() {
        let (file, _source) = parse(
            "pub fn a() {}
pub fn b() {}",
        );
        assert_eq!(collect_pub_like_top_level_fns(&file).len(), 2);
    }

    #[test]
    fn test_usage_pub_and_pub_crate_flagged() {
        let (file, _source) = parse(
            "pub fn a() {}
pub(crate) fn b() {}",
        );
        assert_eq!(collect_pub_like_top_level_fns(&file).len(), 2);
    }

    #[test]
    fn test_usage_one_pub_plus_annotated_private_helpers_is_clean() {
        let (file, source) = parse(
            "pub fn a() {}
// needed helper: x
fn h1() {}
// needed helper: y
fn h2() {}",
        );
        assert_eq!(collect_pub_like_top_level_fns(&file).len(), 1);
        assert_eq!(count_unannotated_private_helpers(&file, &source), 0);
    }

    #[test]
    fn test_usage_single_pub_fn_alone_is_clean() {
        let (file, _source) = parse("pub fn a() {}");
        assert_eq!(collect_pub_like_top_level_fns(&file).len(), 1);
    }
}

// no test_usage necessary
