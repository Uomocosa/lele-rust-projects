use derive_more::{Deref, DerefMut};
use syn::spanned::Spanned;
use syn::visit::Visit;

use lele_lint::diagnostic::Diagnostic;
use lele_lint::entry_kind::EntryKind;
use lele_lint::project::Project;
use lele_lint::severity::Severity;

use super::bevy_ui::BevyUi;

const VISUAL_IDENTS: [&str; 9] = [
    "Sprite",
    "Text2d",
    "Text",
    "Mesh2d",
    "MeshMaterial2d",
    "Camera2d",
    "Camera",
    "Node",
    "ImageNode",
];

pub(crate) fn check(_self: &BevyUi, project: &Project) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    for (rel_path, file) in &project.parsed_files {
        let mut found = Vec::new();
        {
            let mut scanner = UiScanner(&mut found);
            for item in &file.items {
                if is_test_module(item) {
                    continue;
                }
                scanner.visit_item(item);
            }
        }
        let Some(found) = found.into_iter().next() else {
            continue;
        };

        let preview = find_ui_png(file);
        let file_path = project
            .entries
            .iter()
            .find(|e| e.relative_path == *rel_path && e.kind == EntryKind::File)
            .map(|e| e.absolute_path.clone())
            .unwrap_or_else(|| project.src_dir.join(rel_path));

        match preview {
            None => diags.push(Diagnostic {
                file: file_path,
                line: found.line,
                col: 0,
                code: "E029".to_string(),
                message: format!(
                    "file spawns Bevy visual component `{}` but defines no ignored `fn ui_png` test; add one rendering this file's scene",
                    found.visual
                ),
                severity: Severity::Error,
            }),
            Some(line) => {
                let stem = rel_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or_default();
                if !shot_names_stem(file, stem) {
                    diags.push(Diagnostic {
                        file: file_path,
                        line,
                        col: 0,
                        code: "E029".to_string(),
                        message: format!(
                            "`fn ui_png` does not reference `{stem}.png`; name the capture file after the source file"
                        ),
                        severity: Severity::Error,
                    });
                }
            }
        }
    }

    diags
}

struct FoundVisual {
    visual: String,
    line: usize,
}

#[derive(Deref, DerefMut)]
struct UiScanner<'a>(&'a mut Vec<FoundVisual>);

impl<'ast> Visit<'ast> for UiScanner<'_> {
    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        if node.method == "spawn" && self.is_empty() {
            let mut collected = Vec::new();
            {
                let mut names = PathNames(&mut collected);
                for arg in &node.args {
                    names.visit_expr(arg);
                }
            }
            if let Some(visual) = collected
                .iter()
                .find(|name| VISUAL_IDENTS.contains(&name.as_str()))
            {
                self.push(FoundVisual {
                    visual: visual.clone(),
                    line: node.method.span().start().line,
                });
            }
        }
        syn::visit::visit_expr_method_call(self, node);
    }
}

#[derive(Deref, DerefMut)]
struct PathNames<'a>(&'a mut Vec<String>);

impl<'ast> Visit<'ast> for PathNames<'_> {
    fn visit_path(&mut self, node: &'ast syn::Path) {
        for segment in &node.segments {
            self.push(segment.ident.to_string());
        }
        syn::visit::visit_path(self, node);
    }
}

// needed helper:
fn is_test_module(item: &syn::Item) -> bool {
    let syn::Item::Mod(module) = item else {
        return false;
    };
    module.attrs.iter().any(|attr| {
        attr.path().is_ident("cfg")
            && attr
                .parse_nested_meta(|meta| {
                    if meta.path.is_ident("test") {
                        Ok(())
                    } else {
                        Err(meta.error("expected test"))
                    }
                })
                .is_ok()
    })
}

// needed helper:
fn find_ui_png(file: &syn::File) -> Option<usize> {
    let mut finder = UiPngFinder(None);
    finder.visit_file(file);
    *finder
}

#[derive(Default, Deref, DerefMut)]
struct UiPngFinder(Option<usize>);

impl<'ast> Visit<'ast> for UiPngFinder {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        if node.sig.ident == "ui_png"
            && node.attrs.iter().any(|attr| {
                attr.path()
                    .segments
                    .last()
                    .is_some_and(|s| s.ident == "ignore")
            })
        {
            **self = Some(node.sig.fn_token.span().start().line);
        }
        syn::visit::visit_item_fn(self, node);
    }
}

// needed helper:
fn shot_names_stem(file: &syn::File, stem: &str) -> bool {
    let mut finder = StemFinder { stem, found: false };
    finder.visit_file(file);
    finder.found
}

struct StemFinder<'a> {
    stem: &'a str,
    found: bool,
}

impl<'ast> Visit<'ast> for StemFinder<'_> {
    fn visit_lit_str(&mut self, node: &'ast syn::LitStr) {
        if node.value() == format!("{}.png", self.stem) {
            self.found = true;
        }
        syn::visit::visit_lit_str(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::{find_ui_png, shot_names_stem};

    fn parse(source: &str) -> syn::File {
        syn::parse_str(source).unwrap()
    }

    #[test]
    fn test_usage() {
        let with_preview = parse(
            "pub struct Spawner;
             pub fn spawn_target(s: &mut Spawner) { s.spawn((Sprite,)); }
             #[cfg(test)] mod tests {
                 #[test] #[ignore = \"headed\"] fn ui_png() {
                     let p = std::path::PathBuf::from(\"x\").join(\"target.png\");
                     assert!(p.exists());
                 }
             }",
        );
        assert_eq!(find_ui_png(&with_preview), Some(4));
        assert!(shot_names_stem(&with_preview, "target"));
        assert!(!shot_names_stem(&with_preview, "other"));

        let without_preview = parse(
            "pub struct Spawner;
             pub fn spawn_cursor(s: &mut Spawner) { s.spawn((Mesh2d,)); }",
        );
        assert_eq!(find_ui_png(&without_preview), None);

        let unignored = parse(
            "#[cfg(test)] mod tests {
                 #[test] fn ui_png() {}
             }",
        );
        assert_eq!(find_ui_png(&unignored), None);
    }
}
