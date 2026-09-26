use std::path::Path;
use std::path::PathBuf;

use syn::spanned::Spanned;
use syn::visit::Visit;

use crate::checkers;
use crate::AllowWhitelistEntry;
use crate::Diagnostic;
use crate::Project;
use crate::Severity;

pub fn check(
    _self: &checkers::no_allow_attributes::NoAllowAttributes,
    project: &Project,
) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    for (rel_path, file) in &project.parsed_files {
        let mut finder = AllowFinder {
            file: project.src_dir.join(rel_path),
            crate_rel: Path::new("src").join(rel_path),
            clippy_allow_whitelist: &project.clippy_allow_whitelist,
            diags: Vec::new(),
        };
        finder.visit_file(file);
        diags.extend(finder.diags);
    }
    diags
}

fn lint_paths(attr: &syn::Attribute) -> Vec<String> {
    attr.parse_args_with(syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated)
        .map(|paths| {
            paths
                .iter()
                .map(|path| {
                    path.segments
                        .iter()
                        .map(|segment| segment.ident.to_string())
                        .collect::<Vec<_>>()
                        .join("::")
                })
                .collect()
        })
        .unwrap_or_default()
}

struct AllowFinder<'a> {
    file: PathBuf,
    crate_rel: PathBuf,
    clippy_allow_whitelist: &'a [AllowWhitelistEntry],
    diags: Vec<Diagnostic>,
}

impl AllowFinder<'_> {
    fn is_whitelisted(&self, lint_paths: &[String]) -> bool {
        if lint_paths.is_empty() {
            return false;
        }
        self.clippy_allow_whitelist.iter().any(|entry| {
            !entry.reason.trim().is_empty()
                && self.crate_rel == Path::new(&entry.file)
                && lint_paths.iter().all(|path| path == &entry.allow)
        })
    }
}

impl<'ast> Visit<'ast> for AllowFinder<'_> {
    fn visit_attribute(&mut self, attr: &'ast syn::Attribute) {
        let kind = if attr.path().is_ident("allow") {
            "allow"
        } else if attr.path().is_ident("expect") {
            "expect"
        } else {
            return;
        };
        let lint_paths = lint_paths(attr);
        if self.is_whitelisted(&lint_paths) {
            return;
        }
        let start = attr.span().start();
        self.diags.push(Diagnostic {
            file: self.file.clone(),
            line: start.line,
            col: start.column,
            code: "E023".to_string(),
            message: format!(
                "`{kind}` attribute is banned — add an [[lele.lint.clippy_allow_whitelist]] entry with the exact `allow` lint path, `file` and a non-empty `reason` in lele.toml to whitelist it"
            ),
            severity: Severity::Error,
        });
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::path::PathBuf;

    use super::check;
    use crate::checkers;
    use crate::checkers::no_allow_attributes::NoAllowAttributes;
    use crate::AllowWhitelistEntry;
    use crate::Project;

    fn entry(allow: &str, file: &str, reason: &str) -> AllowWhitelistEntry {
        AllowWhitelistEntry {
            allow: allow.to_string(),
            file: file.to_string(),
            reason: reason.to_string(),
        }
    }

    #[test]
    fn test_usage() {
        let checker = checkers::no_allow_attributes::NoAllowAttributes;
        let project = Project::default();
        assert!(check(&checker, &project).is_empty());
    }

    fn run_check_with(code: &str, clippy_allow_whitelist: Vec<AllowWhitelistEntry>) -> usize {
        let file: syn::File = syn::parse_str(code).unwrap();
        let mut parsed_files = HashMap::new();
        parsed_files.insert(PathBuf::from("x.rs"), file);
        let project = Project {
            root: PathBuf::from("."),
            src_dir: PathBuf::from("src"),
            entries: Vec::new(),
            module_info: HashMap::default(),
            parsed_files,
            clippy_allow_whitelist,
            ..Project::default()
        };
        check(&NoAllowAttributes, &project).len()
    }

    fn run_check(code: &str) -> usize {
        run_check_with(code, Vec::new())
    }

    #[test]
    fn test_usage_flags_allow() {
        assert_eq!(run_check("#[allow(dead_code)] struct Foo;"), 1);
        assert_eq!(run_check("struct Foo { #[allow(dead_code)] bar: u32 }"), 1);
        assert_eq!(run_check("#![allow(dead_code)]"), 1);
        assert_eq!(
            run_check("#[allow(clippy::missing_const_for_fn)] fn f() {}"),
            1
        );
    }

    #[test]
    fn test_usage_flags_expect() {
        assert_eq!(run_check("#[expect(dead_code)] struct Foo;"), 1);
    }

    #[test]
    fn test_usage_clean_passes() {
        assert_eq!(run_check("struct Foo { bar: u32 }"), 0);
        assert_eq!(run_check("#[derive(Clone)] struct Foo { bar: u32 }"), 0);
        assert_eq!(run_check("#[cfg(test)] mod tests {}"), 0);
    }

    #[test]
    fn test_usage_whitelisted_passes() {
        let whitelist = vec![entry("unreachable_code", "src/x.rs", "derive macro")];
        assert_eq!(run_check_with("#![allow(unreachable_code)]", whitelist), 0);
    }

    #[test]
    fn test_usage_whitelist_needs_matching_file() {
        let whitelist = vec![entry("unreachable_code", "src/other.rs", "derive macro")];
        assert_eq!(run_check_with("#![allow(unreachable_code)]", whitelist), 1);
    }

    #[test]
    fn test_usage_whitelist_needs_reason() {
        let whitelist = vec![entry("unreachable_code", "src/x.rs", "   ")];
        assert_eq!(run_check_with("#![allow(unreachable_code)]", whitelist), 1);
    }

    #[test]
    fn test_usage_whitelist_lists_all_lints() {
        let whitelist = vec![entry("unreachable_code", "src/x.rs", "derive macro")];
        assert_eq!(
            run_check_with(
                "#[allow(unreachable_code, dead_code)] struct Foo;",
                whitelist
            ),
            1
        );
    }
}
