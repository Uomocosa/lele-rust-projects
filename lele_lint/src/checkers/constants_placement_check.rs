use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use syn::visit::Visit;

use super::constants_placement::ConstantsPlacement;
use crate::Diagnostic;
use crate::Project;
use crate::Severity;

pub(crate) fn check(_self: &ConstantsPlacement, project: &Project) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    let defined = collect_defined_consts(&project.parsed_files);
    let defined_names: HashSet<String> = defined.keys().cloned().collect();
    let usages = collect_const_usages(&project.parsed_files, &defined_names);

    for (name, rel_path) in &defined {
        if let Some(users) = usages.get(name) {
            check_fn_file_const(name, rel_path, project, &mut diags);
            check_lift_to_ancestor(name, rel_path, users, project, &mut diags);
            check_lower_to_subdir(name, rel_path, users, project, &mut diags);
        }
    }

    diags
}

// needed helper: `pub`/`pub(crate)` const/static definition collection
fn collect_defined_consts(parsed_files: &HashMap<PathBuf, syn::File>) -> HashMap<String, PathBuf> {
    let mut out = HashMap::new();
    for (rel_path, file) in parsed_files {
        for item in &file.items {
            let (name, exposed) = match item {
                syn::Item::Const(c) => (c.ident.to_string(), is_exposed(&c.vis)),
                syn::Item::Static(s) => (s.ident.to_string(), is_exposed(&s.vis)),
                _ => continue,
            };
            if exposed {
                out.insert(name, rel_path.clone());
            }
        }
    }
    out
}

// needed helper: per-const user-file collection for `module::CONST` paths
fn collect_const_usages(
    parsed_files: &HashMap<PathBuf, syn::File>,
    defined: &HashSet<String>,
) -> HashMap<String, HashSet<PathBuf>> {
    let mut usages: HashMap<String, HashSet<PathBuf>> = HashMap::new();
    for (rel_path, file) in parsed_files {
        let mut visitor = ConstUseVisitor {
            defined,
            found: Vec::new(),
        };
        visitor.visit_file(file);
        for name in visitor.found {
            usages.entry(name).or_default().insert(rel_path.clone());
        }
    }
    usages
}

// needed helper: AST visitor collecting trailing-segment const names
struct ConstUseVisitor<'a> {
    defined: &'a HashSet<String>,
    found: Vec<String>,
}

impl<'ast> Visit<'ast> for ConstUseVisitor<'_> {
    fn visit_path(&mut self, path: &'ast syn::Path) {
        if let Some(last) = path.segments.last() {
            let name = last.ident.to_string();
            if self.defined.contains(&name) && path.segments.len() >= 2 {
                self.found.push(name);
            }
        }
        syn::visit::visit_path(self, path);
    }
}

// needed helper: const-in-fn-file detection (must live in sibling constants.rs)
fn check_fn_file_const(
    name: &str,
    rel_path: &Path,
    project: &Project,
    diags: &mut Vec<Diagnostic>,
) {
    let file_name = rel_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    if file_name == "constants.rs" || file_name == "mod.rs" || file_name == "lib.rs" {
        return;
    }
    let Some(file) = project.parsed_files.get(rel_path) else {
        return;
    };
    let has_pub_fn = file
        .items
        .iter()
        .any(|item| matches!(item, syn::Item::Fn(f) if is_exposed(&f.vis)));
    if has_pub_fn {
        let sibling = sibling_constants(rel_path);
        diags.push(Diagnostic {
            file: project.src_dir.join(rel_path),
            line: 1,
            col: 0,
            code: "E026".to_string(),
            message: format!(
                "const `{name}` lives in fn-file `{}` — move it to `{}` (nearest constants.rs)",
                rel_path.display(),
                sibling.display(),
            ),
            severity: Severity::Error,
        });
    }
}

// needed helper: lift detection when users span top-level dirs
fn check_lift_to_ancestor(
    name: &str,
    rel_path: &Path,
    users: &HashSet<PathBuf>,
    project: &Project,
    diags: &mut Vec<Diagnostic>,
) {
    let ancestor = common_ancestor_dir(users);
    let def_dir = parent_dir(rel_path);
    if ancestor.is_empty() && !def_dir.is_empty() {
        diags.push(Diagnostic {
            file: project.src_dir.join(rel_path),
            line: 1,
            col: 0,
            code: "E026".to_string(),
            message: format!(
                "const `{name}` is used across top-level dirs but lives in `{}` — lift it to `src/constants.rs`",
                rel_path.display(),
            ),
            severity: Severity::Error,
        });
    }
}

// needed helper: lower detection when root const is used in one subdir only
fn check_lower_to_subdir(
    name: &str,
    rel_path: &Path,
    users: &HashSet<PathBuf>,
    project: &Project,
    diags: &mut Vec<Diagnostic>,
) {
    if rel_path.components().count() != 1 {
        return;
    }
    let ancestor = common_ancestor_dir(users);
    if ancestor.as_os_str().is_empty() {
        return;
    }
    diags.push(Diagnostic {
        file: project.src_dir.join(rel_path),
        line: 1,
        col: 0,
        code: "E026".to_string(),
        message: format!(
            "const `{name}` lives at root but is only used under `{}` — lower it to `{}/constants.rs`",
            ancestor.display(),
            ancestor.display(),
        ),
        severity: Severity::Error,
    });
}

// needed helper: sibling constants.rs path for a file
fn sibling_constants(rel_path: &Path) -> PathBuf {
    let parent = parent_dir(rel_path);
    if parent.as_os_str().is_empty() {
        PathBuf::from("constants.rs")
    } else {
        parent.join("constants.rs")
    }
}

// needed helper: parent dir of a relative path
fn parent_dir(rel_path: &Path) -> PathBuf {
    rel_path.parent().map(Path::to_path_buf).unwrap_or_default()
}

// needed helper: deepest common ancestor dir of user files
fn common_ancestor_dir(users: &HashSet<PathBuf>) -> PathBuf {
    let mut iter = users.iter();
    let Some(first) = iter.next() else {
        return PathBuf::new();
    };
    let mut prefix: PathBuf = parent_dir(first);
    for other in iter {
        let other_dir = parent_dir(other);
        prefix = common_prefix(&prefix, &other_dir);
    }
    prefix
}

// needed helper: common path prefix of two dirs
fn common_prefix(a: &Path, b: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for (x, y) in a.components().zip(b.components()) {
        if x == y {
            out.push(x);
        } else {
            break;
        }
    }
    out
}

// needed helper: `pub` or `pub(crate)` visibility check
fn is_exposed(vis: &syn::Visibility) -> bool {
    match vis {
        syn::Visibility::Public(_) => true,
        syn::Visibility::Restricted(r) => {
            r.path.segments.len() == 1
                && r.path.segments.first().is_some_and(|s| s.ident == "crate")
        }
        syn::Visibility::Inherited => false,
    }
}

#[cfg(test)]
mod tests {
    use super::{common_ancestor_dir, common_prefix, sibling_constants};
    use std::collections::HashSet;
    use std::path::PathBuf;

    fn set(paths: &[&str]) -> HashSet<PathBuf> {
        paths.iter().map(PathBuf::from).collect()
    }

    #[test]
    fn test_usage_sibling_constants() {
        assert_eq!(
            sibling_constants(&PathBuf::from("discover.rs")),
            PathBuf::from("constants.rs")
        );
        assert_eq!(
            sibling_constants(&PathBuf::from("checkers/foo.rs")),
            PathBuf::from("checkers/constants.rs")
        );
    }

    #[test]
    fn test_usage_common_prefix() {
        assert_eq!(
            common_prefix(
                &PathBuf::from("checkers/sub"),
                &PathBuf::from("checkers/other")
            ),
            PathBuf::from("checkers")
        );
        assert_eq!(
            common_prefix(&PathBuf::from("checkers"), &PathBuf::from("project")),
            PathBuf::new()
        );
    }

    #[test]
    fn test_usage_common_ancestor_single_dir() {
        let users = set(&["checkers/a.rs", "checkers/b.rs"]);
        assert_eq!(common_ancestor_dir(&users), PathBuf::from("checkers"));
    }

    #[test]
    fn test_usage_common_ancestor_spans_top() {
        let users = set(&["checkers/a.rs", "project/b.rs"]);
        assert_eq!(common_ancestor_dir(&users), PathBuf::new());
    }

    #[test]
    fn test_usage_common_ancestor_lifts_nested() {
        let users = set(&["checkers/sub/a.rs", "checkers/other/b.rs"]);
        assert_eq!(common_ancestor_dir(&users), PathBuf::from("checkers"));
    }
}
