// no test_usage necessary

// no test_usage necessary
// needed helper: parsing utilities

use std::path::Path;

use super::helper_count::HelperCount;
use crate::diagnostic::Diagnostic;
use crate::entry_kind::EntryKind;
use crate::project::Project;
use crate::severity::Severity;

const MAX_HELPERS: usize = 2;

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

        if has_opt_out(project, rel_path) {
            continue;
        }

        let helper_count = count_top_level_helpers(file);

        if helper_count > MAX_HELPERS {
            diags.push(Diagnostic {
                file: project.src_dir.join(rel_path),
                line: 1,
                col: 0,
                code: "E015".to_string(),
                message: format!(
                    "{} helper functions (max {}). Extract pure/reusable ones as thin delegates; keep only context-specific ones with `// needed helper:` to justify",
                    helper_count,
                    MAX_HELPERS
                ),
                severity: Severity::Warning,
            });
        }
    }

    diags
}

fn has_opt_out(project: &Project, rel_path: &Path) -> bool {
    let entry = match project
        .entries
        .iter()
        .find(|e| e.relative_path == rel_path && e.kind == EntryKind::File)
    {
        Some(e) => e,
        None => return false,
    };

    let content = match std::fs::read_to_string(&entry.absolute_path) {
        Ok(c) => c,
        Err(_) => return false,
    };

    content
        .lines()
        .any(|line| line.trim().starts_with("// needed helper:"))
}

fn count_top_level_helpers(file: &syn::File) -> usize {
    file.items
        .iter()
        .filter(|item| {
            if let syn::Item::Fn(func) = item {
                return !matches!(func.vis, syn::Visibility::Public(_));
            }
            false
        })
        .count()
}

#[cfg(test)]
mod tests {
    use super::count_top_level_helpers;

    #[test]
    fn test_usage_counts_helpers() {
        let file: syn::File = syn::parse_str(
            "pub struct Foo;
            pub fn bar() {}
            fn helper_a() {}
            fn helper_b() {}
            fn helper_c() {}
            #[cfg(test)] mod tests { fn test_usage() {} }",
        )
        .unwrap();
        assert_eq!(count_top_level_helpers(&file), 3);
    }

    #[test]
    fn test_usage_skips_impl_fns() {
        let file: syn::File = syn::parse_str(
            "pub struct Foo;
            impl Foo { fn method_a() {} fn method_b() {} fn method_c() {} }
            fn helper_a() {}",
        )
        .unwrap();
        assert_eq!(count_top_level_helpers(&file), 1);
    }
}
