use std::path::PathBuf;

use syn::spanned::Spanned;
use syn::visit::Visit;

use super::no_allow_attributes::NoAllowAttributes;
use crate::Diagnostic;
use crate::Project;
use crate::Severity;

pub(crate) fn check(_self: &NoAllowAttributes, project: &Project) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    for (rel_path, file) in &project.parsed_files {
        let mut finder = AllowFinder {
            file: project.src_dir.join(rel_path),
            diags: Vec::new(),
        };
        finder.visit_file(file);
        diags.extend(finder.diags);
    }
    diags
}

struct AllowFinder {
    file: PathBuf,
    diags: Vec<Diagnostic>,
}

impl<'ast> Visit<'ast> for AllowFinder {
    fn visit_attribute(&mut self, attr: &'ast syn::Attribute) {
        let kind = if attr.path().is_ident("allow") {
            "allow"
        } else if attr.path().is_ident("expect") {
            "expect"
        } else {
            return;
        };
        let start = attr.span().start();
        self.diags.push(Diagnostic {
            file: self.file.clone(),
            line: start.line,
            col: start.column,
            code: "E023".to_string(),
            message: format!(
                "`{kind}` attribute is banned — remove it or set `no_allow_attributes = false` for this crate in lele.toml"
            ),
            severity: Severity::Error,
        });
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::path::PathBuf;

    use super::super::no_allow_attributes::NoAllowAttributes;
    use super::check;
    use crate::Project;

    fn run_check(code: &str) -> usize {
        let file: syn::File = syn::parse_str(code).unwrap();
        let mut parsed_files = HashMap::new();
        parsed_files.insert(PathBuf::from("x.rs"), file);
        let project = Project {
            root: PathBuf::from("."),
            src_dir: PathBuf::from("src"),
            entries: Vec::new(),
            module_info: HashMap::default(),
            parsed_files,
        };
        check(&NoAllowAttributes, &project).len()
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
}
