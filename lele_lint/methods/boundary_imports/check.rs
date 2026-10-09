use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;

use syn::spanned::Spanned;
use syn::visit::Visit;

use crate::checkers;
use crate::BoundaryEntry;
use crate::Diagnostic;
use crate::Project;

pub fn check(
    _self: &checkers::boundary_imports::BoundaryImports,
    project: &Project,
) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    if project.boundaries.is_empty() {
        return diags;
    }
    let stem_map = src_stem_map(project);
    let files = collect_file_uses(project);
    for boundary in &project.boundaries {
        validate_boundary(project, boundary, &mut diags);
        if boundary.cannot_use.is_empty() {
            continue;
        }
        for file in &files {
            if !file_in_boundary(&file.crate_rel, &boundary.folders, &stem_map) {
                continue;
            }
            report_uses(file, boundary, &mut diags);
        }
    }
    diags
}

// needed helper: map a source file stem to its src-relative path
fn src_stem_map(project: &Project) -> HashMap<String, PathBuf> {
    project
        .parsed_files
        .keys()
        .filter_map(|rel| {
            rel.file_stem()
                .and_then(|stem| stem.to_str())
                .map(|stem| (stem.to_string(), rel.clone()))
        })
        .collect()
}

// needed helper: a file is inside a boundary directly or through its primary type
fn file_in_boundary(
    crate_rel: &Path,
    folders: &[PathBuf],
    stem_map: &HashMap<String, PathBuf>,
) -> bool {
    if folders.iter().any(|folder| crate_rel.starts_with(folder)) {
        return true;
    }
    let Ok(rest) = crate_rel.strip_prefix("methods") else {
        return false;
    };
    let Some(type_dir) = rest
        .components()
        .next()
        .and_then(|c| c.as_os_str().to_str())
    else {
        return false;
    };
    let Some(src_rel) = stem_map.get(type_dir) else {
        return false;
    };
    let src_crate_rel = Path::new("src").join(src_rel);
    folders
        .iter()
        .any(|folder| src_crate_rel.starts_with(folder))
}

// needed helper: validate the boundary declaration itself
fn validate_boundary(project: &Project, boundary: &BoundaryEntry, diags: &mut Vec<Diagnostic>) {
    if boundary.folders.is_empty() {
        diags.push(config_diag(
            project,
            &format!("boundary `{}` has no folders", boundary.name),
        ));
    }
    for folder in &boundary.folders {
        if !project.root.join(folder).is_dir() {
            diags.push(config_diag(
                project,
                &format!(
                    "boundary `{}` names folder `{}` which does not exist",
                    boundary.name,
                    folder.display()
                ),
            ));
        }
    }
    if boundary.cannot_use.is_empty() && boundary.require.is_none() {
        diags.push(config_diag(
            project,
            &format!(
                "boundary `{}` needs a non-empty `cannot_use` or a `require`",
                boundary.name
            ),
        ));
    }
}

// needed helper: a boundary config error is reported on `lele.toml` line 1
fn config_diag(project: &Project, message: &str) -> Diagnostic {
    Diagnostic {
        file: project.root.join("lele.toml"),
        line: 1,
        col: 0,
        code: checkers::boundary_imports::BoundaryImports::CODE.to_string(),
        message: message.to_string(),
    }
}

// needed helper: one path use (import or expression path) in a file
struct PathHit {
    path: String,
    line: usize,
    glob: bool,
}

// needed helper: a file's uses and the absolute path to report them on
struct FileUse {
    file: PathBuf,
    crate_rel: PathBuf,
    hits: Vec<PathHit>,
}

// needed helper: collect every use and path from all in-scope files
fn collect_file_uses(project: &Project) -> Vec<FileUse> {
    let mut files = Vec::new();
    for (rel, parsed) in &project.parsed_files {
        files.push(file_use(
            project.src_dir.join(rel),
            Path::new("src").join(rel),
            parsed,
        ));
    }
    if let Some(methods_dir) = &project.methods_dir {
        for (rel, parsed) in &project.methods_parsed_files {
            files.push(file_use(
                methods_dir.join(rel),
                Path::new("methods").join(rel),
                parsed,
            ));
        }
    }
    files
}

// needed helper: two passes over one file (aliases then path expansion)
fn file_use(file: PathBuf, crate_rel: PathBuf, parsed: &syn::File) -> FileUse {
    let mut uses = UseCollector::default();
    uses.visit_file(parsed);
    let mut paths = PathCollector {
        aliases: uses.aliases.clone(),
        hits: Vec::new(),
    };
    paths.visit_file(parsed);
    let mut hits = uses.hits;
    hits.extend(paths.hits);
    FileUse {
        file,
        crate_rel,
        hits,
    }
}

// needed helper: report every forbidden path used by a file
fn report_uses(file: &FileUse, boundary: &BoundaryEntry, diags: &mut Vec<Diagnostic>) {
    for hit in &file.hits {
        if is_local(&hit.path) {
            continue;
        }
        let Some(reported) = forbidden_hit(hit, &boundary.cannot_use) else {
            continue;
        };
        diags.push(use_diag(&file.file, hit.line, &reported, boundary));
    }
}

// needed helper: build the diagnostic message for one forbidden use
fn use_diag(file: &Path, line: usize, reported: &str, boundary: &BoundaryEntry) -> Diagnostic {
    let name = &boundary.name;
    let folders = boundary
        .folders
        .iter()
        .map(|folder| folder.display().to_string())
        .collect::<Vec<_>>()
        .join(", ");
    let why = &boundary.why;
    Diagnostic {
        file: file.to_path_buf(),
        line,
        col: 0,
        code: checkers::boundary_imports::BoundaryImports::CODE.to_string(),
        message: format!("`{reported}` is not allowed in boundary \"{name}\" ({folders}) — {why}"),
    }
}

// needed helper: `crate`, `self`, `super` paths are local, never forbidden
fn is_local(path: &str) -> bool {
    ["crate", "self", "super"]
        .iter()
        .any(|root| path == *root || path.starts_with(&format!("{root}::")))
}

// needed helper: first forbidden list entry matched by one hit
fn forbidden_hit(hit: &PathHit, cannot_use: &[String]) -> Option<String> {
    if cannot_use
        .iter()
        .any(|forbidden| matches_hit(forbidden, hit))
    {
        Some(hit.path.clone())
    } else {
        None
    }
}

// needed helper: segment-wise match; a glob also covers its descendants
fn matches_hit(forbidden: &str, hit: &PathHit) -> bool {
    let prefix = format!("{}::", hit.path);
    if hit.glob {
        return forbidden == hit.path || forbidden.starts_with(&prefix);
    }
    forbidden == hit.path || hit.path.starts_with(&format!("{forbidden}::"))
}

// needed helper: collect use-item aliases and report the imported leaves
#[derive(Default)]
struct UseCollector {
    aliases: HashMap<String, String>,
    hits: Vec<PathHit>,
}

impl<'ast> Visit<'ast> for UseCollector {
    fn visit_item_use(&mut self, node: &'ast syn::ItemUse) {
        let line = node.span().start().line;
        let mut prefix = Vec::new();
        walk_use_tree(&node.tree, &mut prefix, self, line);
    }
}

// needed helper: flatten a use tree into aliases and leaf hits
fn walk_use_tree(
    tree: &syn::UseTree,
    prefix: &mut Vec<String>,
    collector: &mut UseCollector,
    line: usize,
) {
    match tree {
        syn::UseTree::Path(path) => {
            let mut next = prefix.clone();
            next.push(path.ident.to_string());
            walk_use_tree(&path.tree, &mut next, collector, line);
        }
        syn::UseTree::Name(name) => {
            let mut full = prefix.clone();
            if name.ident != "self" {
                full.push(name.ident.to_string());
            }
            let joined = full.join("::");
            collector
                .aliases
                .insert(name.ident.to_string(), joined.clone());
            collector.hits.push(PathHit {
                path: joined,
                line,
                glob: false,
            });
        }
        syn::UseTree::Rename(rename) => {
            let mut full = prefix.clone();
            full.push(rename.ident.to_string());
            let joined = full.join("::");
            collector
                .aliases
                .insert(rename.rename.to_string(), joined.clone());
            collector.hits.push(PathHit {
                path: joined,
                line,
                glob: false,
            });
        }
        syn::UseTree::Glob(_) => {
            collector.hits.push(PathHit {
                path: prefix.join("::"),
                line,
                glob: true,
            });
        }
        syn::UseTree::Group(group) => {
            for item in &group.items {
                walk_use_tree(item, prefix, collector, line);
            }
        }
    }
}

// needed helper: expand and report every expression/type path in the file
struct PathCollector {
    aliases: HashMap<String, String>,
    hits: Vec<PathHit>,
}

impl<'ast> Visit<'ast> for PathCollector {
    fn visit_path(&mut self, path: &'ast syn::Path) {
        let raw = path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect::<Vec<_>>()
            .join("::");
        self.hits.push(PathHit {
            path: expand(&raw, &self.aliases),
            line: path.span().start().line,
            glob: false,
        });
        syn::visit::visit_path(self, path);
    }
}

// needed helper: rewrite a leading use alias to its full path
fn expand(raw: &str, aliases: &HashMap<String, String>) -> String {
    let Some((first, rest)) = raw.split_once("::") else {
        return aliases.get(raw).cloned().unwrap_or_else(|| raw.to_string());
    };
    match aliases.get(first) {
        Some(full) if rest.is_empty() => full.clone(),
        Some(full) => format!("{full}::{rest}"),
        None => raw.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{expand, is_local, matches_hit, PathHit};

    #[test]
    fn test_usage() {
        assert!(is_local("crate::foo"));
        assert!(is_local("self::foo"));
        assert!(is_local("super::foo"));
        assert!(!is_local("std::time"));

        let direct = PathHit {
            path: "std::time::Instant".to_string(),
            line: 1,
            glob: false,
        };
        assert!(matches_hit("std::time", &direct));
        assert!(!matches_hit("std::timex", &direct));

        let glob = PathHit {
            path: "std::time".to_string(),
            line: 1,
            glob: true,
        };
        assert!(matches_hit("std::time::Instant", &glob));

        let mut aliases = HashMap::new();
        aliases.insert("time".to_string(), "std::time".to_string());
        assert_eq!(
            expand("time::Instant::now", &aliases),
            "std::time::Instant::now"
        );
        assert_eq!(expand("time", &aliases), "std::time");
        assert_eq!(expand("std::fs::read", &aliases), "std::fs::read");
    }
}
