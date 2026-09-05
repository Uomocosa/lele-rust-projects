use std::collections::HashSet;
use std::path::Path;

use syn::spanned::Spanned;
use syn::visit::Visit;

use super::no_stuttered_path::NoStutteredPath;
use crate::common;
use crate::Diagnostic;
use crate::Project;
use crate::Severity;

pub(crate) fn check(_self: &NoStutteredPath, project: &Project) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    let root_stems = root_module_stems(project);

    for (rel_path, file) in &project.parsed_files {
        let mut hits = Vec::new();
        {
            let mut visitor = StutterVisitor {
                hits: &mut hits,
                stems: &root_stems,
            };
            visitor.visit_file(file);
        }
        for hit in std::mem::take(&mut hits) {
            diags.push(Diagnostic {
                file: project.src_dir.join(rel_path),
                line: hit.line,
                col: 0,
                code: NoStutteredPath::CODE.to_string(),
                message: format!(
                    "stuttered path `{}` adds no information — import `{}` once and use `{}` directly",
                    hit.path, hit.ty, hit.ty
                ),
                severity: Severity::Error,
            });
        }
    }

    diags
}

// needed helper: root `src/*.rs` file stems — only these modules flatten to `crate::Type`
fn root_module_stems(project: &Project) -> HashSet<String> {
    project
        .parsed_files
        .keys()
        .filter(|rel| rel.components().count() == 1)
        .filter_map(|rel| stem_name(rel))
        .collect()
}

// needed helper: file stem of a relative path
fn stem_name(rel_path: &Path) -> Option<String> {
    rel_path
        .file_stem()
        .and_then(|s| s.to_str())
        .map(str::to_string)
}

// needed helper: AST visitor collecting `module::Type` paths where the type repeats the module
struct StutterHit {
    line: usize,
    path: String,
    ty: String,
}

struct StutterVisitor<'a> {
    hits: &'a mut Vec<StutterHit>,
    stems: &'a HashSet<String>,
}

impl<'ast> Visit<'ast> for StutterVisitor<'_> {
    fn visit_path(&mut self, path: &'ast syn::Path) {
        collect_stutter(path, self.stems, self.hits);
        syn::visit::visit_path(self, path);
    }
}

// needed helper: adjacent-segment stutter scan for a single path
fn collect_stutter(path: &syn::Path, root_stems: &HashSet<String>, hits: &mut Vec<StutterHit>) {
    if path.leading_colon.is_some() {
        return;
    }
    let segments: Vec<String> = path.segments.iter().map(|s| s.ident.to_string()).collect();
    if segments.first().is_some_and(|s| s == "crate") {
        return;
    }
    for pair in segments.windows(2) {
        if let [module, ty] = pair {
            if root_stems.contains(module) && common::is_stuttered_path(module, ty) {
                hits.push(StutterHit {
                    line: path.span().start().line,
                    path: segments.join("::"),
                    ty: ty.clone(),
                });
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::path::PathBuf;
    use syn::visit::Visit;

    use super::collect_stutter;

    struct Collector<'a> {
        hits: &'a mut Vec<super::StutterHit>,
        stems: &'a HashSet<String>,
    }

    impl<'ast> Visit<'ast> for Collector<'_> {
        fn visit_path(&mut self, path: &'ast syn::Path) {
            collect_stutter(path, self.stems, self.hits);
            syn::visit::visit_path(self, path);
        }
    }

    fn stems(names: &[&str]) -> HashSet<String> {
        names.iter().map(|s| (*s).to_string()).collect()
    }

    fn hit_paths(src: &str, stems: &HashSet<String>) -> Vec<String> {
        let file = syn::parse_file(src).unwrap();
        let mut hits = Vec::new();
        {
            let mut visitor = Collector {
                hits: &mut hits,
                stems,
            };
            visitor.visit_file(&file);
        }
        std::mem::take(&mut hits)
            .into_iter()
            .map(|hit| hit.path)
            .collect()
    }

    #[test]
    fn test_usage_flags_diagnostic_stutter() {
        let stems = stems(&["diagnostic"]);
        let src = "use crate::diagnostic;\npub fn f(d: Vec<diagnostic::Diagnostic>) {}\n";
        assert_eq!(hit_paths(src, &stems), vec!["diagnostic::Diagnostic"]);
    }

    #[test]
    fn test_usage_flags_severity_stutter() {
        let stems = stems(&["severity"]);
        let src = "fn f(s: severity::Severity) {}\n";
        assert_eq!(hit_paths(src, &stems), vec!["severity::Severity"]);
    }

    #[test]
    fn test_usage_flags_stutter_prefix_of_longer_path() {
        let stems = stems(&["entry_kind"]);
        let src = "fn f() { let _ = entry_kind::EntryKind::File; }\n";
        assert_eq!(hit_paths(src, &stems), vec!["entry_kind::EntryKind::File"]);
    }

    #[test]
    fn test_usage_allows_distinct_names() {
        let stems = stems(&["diagnostic"]);
        let src = "use crate::boxes;\nfn f(id: boxes::PlayerId) {}\nfn g(m: module_info::ModuleInfoMap) {}\n";
        assert_eq!(hit_paths(src, &stems), Vec::<String>::new());
    }

    #[test]
    fn test_usage_allows_subfolder_domain_prefix() {
        let stems = stems(&["player_id"]);
        let src = "use crate::player;\nfn f() { let _ = player::Player::new(); }\n";
        assert_eq!(hit_paths(src, &stems), Vec::<String>::new());
    }

    #[test]
    fn test_usage_flags_root_module_stutter_only() {
        let without = HashSet::new();
        let src = "fn f() { let _ = player::Player::new(); }\n";
        assert_eq!(hit_paths(src, &without), Vec::<String>::new());
        let with = stems(&["player"]);
        assert_eq!(hit_paths(src, &with), vec!["player::Player::new"]);
    }

    #[test]
    fn test_usage_allows_function_dispatch() {
        let stems = stems(&["config_new"]);
        let src = "fn f() { config_new::new(); }\n";
        assert_eq!(hit_paths(src, &stems), Vec::<String>::new());
    }

    #[test]
    fn test_usage_allows_external_paths() {
        let stems = stems(&["toml", "syn"]);
        let src = "fn f(v: toml::Value) {}\nfn g(item: syn::ItemMod) {}\n";
        assert_eq!(hit_paths(src, &stems), Vec::<String>::new());
    }

    #[test]
    fn test_usage_allows_use_items_and_crate_paths() {
        let stems = stems(&["diagnostic"]);
        let src = "use crate::diagnostic::Diagnostic;\nfn f(d: crate::diagnostic::Diagnostic) {}\n";
        assert_eq!(hit_paths(src, &stems), Vec::<String>::new());
    }

    #[test]
    fn test_usage_stem_name_from_rel_path() {
        use super::stem_name;
        assert_eq!(
            stem_name(&PathBuf::from("diagnostic.rs")),
            Some("diagnostic".to_string())
        );
        assert_eq!(
            stem_name(&PathBuf::from("checkers/no_crate_paths.rs")),
            Some("no_crate_paths".to_string())
        );
    }
}
