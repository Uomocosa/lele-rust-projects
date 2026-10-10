use derive_more::{Deref, DerefMut};
use syn::spanned::Spanned;
use syn::visit::Visit;

use crate::scan;

const PNG_SUFFIX: &str = "_ui_png_preview";
const MP4_SUFFIX: &str = "_ui_mp4_preview";
const SCENE_SUFFIX: &str = "_ui_scene_preview";
const PREVIEW_CRATE: &str = "lele_bevy_preview";
const RUN_FN: &str = "run";

#[must_use]
pub fn collect_previews(file: &syn::File) -> Vec<scan::Preview> {
    let uses_crate = mentions_preview_crate(file);
    let mut previews = Vec::new();
    let mut finder = PreviewFinder {
        previews: &mut previews,
        uses_crate,
    };
    finder.visit_file(file);
    previews
}

// needed helper: matches one preview fn into a Preview record
struct PreviewFinder<'a> {
    previews: &'a mut Vec<scan::Preview>,
    uses_crate: bool,
}

impl<'ast> Visit<'ast> for PreviewFinder<'_> {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        let name = node.sig.ident.to_string();
        if let Some(kind) = kind_of(&name) {
            let mut idents = Vec::new();
            let mut runs = false;
            {
                let mut body = BodyScanner {
                    idents: &mut idents,
                    runs: &mut runs,
                };
                body.visit_block(&node.block);
            }
            self.previews.push(scan::Preview {
                name,
                kind,
                line: node.sig.fn_token.span().start().line,
                is_ignored: has_ignored(&node.attrs),
                is_routed_through_harness: runs && self.uses_crate,
                idents,
            });
        }
        syn::visit::visit_item_fn(self, node);
    }
}

// needed helper: preview kind from the fn-name suffix
fn kind_of(name: &str) -> Option<scan::PreviewKind> {
    if name.ends_with(PNG_SUFFIX) {
        Some(scan::PreviewKind::Png)
    } else if name.ends_with(MP4_SUFFIX) {
        Some(scan::PreviewKind::Mp4)
    } else if name.ends_with(SCENE_SUFFIX) {
        Some(scan::PreviewKind::Scene)
    } else {
        None
    }
}

// needed helper: `#[ignore]` presence on a test fn
fn has_ignored(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path()
            .segments
            .last()
            .is_some_and(|segment| segment.ident == "ignore")
    })
}

// needed helper: does the file name `lele_bevy_preview` anywhere (use or path)?
fn mentions_preview_crate(file: &syn::File) -> bool {
    let mut found = false;
    let mut visitor = CrateMention(&mut found);
    visitor.visit_file(file);
    found
}

// needed helper: scans paths and use-trees for the harness crate name
#[derive(Deref, DerefMut)]
struct CrateMention<'a>(&'a mut bool);

impl<'ast> Visit<'ast> for CrateMention<'_> {
    fn visit_path(&mut self, node: &'ast syn::Path) {
        if node.segments.iter().any(|s| s.ident == PREVIEW_CRATE) {
            ***self = true;
        }
        syn::visit::visit_path(self, node);
    }

    fn visit_use_tree(&mut self, node: &'ast syn::UseTree) {
        match node {
            syn::UseTree::Path(path) => {
                if path.ident == PREVIEW_CRATE {
                    ***self = true;
                }
                self.visit_use_tree(&path.tree);
            }
            syn::UseTree::Name(name) => {
                if name.ident == PREVIEW_CRATE {
                    ***self = true;
                }
            }
            syn::UseTree::Rename(rename) => {
                if rename.ident == PREVIEW_CRATE {
                    ***self = true;
                }
            }
            syn::UseTree::Group(group) => {
                for tree in &group.items {
                    self.visit_use_tree(tree);
                }
            }
            syn::UseTree::Glob(_) => {}
        }
    }
}

// needed helper: collects path idents and detects a `run(...)` call in a preview body
struct BodyScanner<'a> {
    idents: &'a mut Vec<String>,
    runs: &'a mut bool,
}

impl<'ast> Visit<'ast> for BodyScanner<'_> {
    fn visit_path(&mut self, node: &'ast syn::Path) {
        for segment in &node.segments {
            self.idents.push(segment.ident.to_string());
        }
        syn::visit::visit_path(self, node);
    }

    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if let syn::Expr::Path(path) = &*node.func
            && path.path.segments.last().is_some_and(|s| s.ident == RUN_FN)
        {
            *self.runs = true;
        }
        syn::visit::visit_expr_call(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::collect_previews;

    #[test]
    fn test_usage() {
        let file = syn::parse_str(
            "use lele_bevy_preview::{run, scene::Scene};
             #[derive(Component)] pub struct Root;
             #[cfg(test)]
             mod tests {
                 #[test]
                 #[ignore = \"headed\"]
                 fn spawn_root_ui_png_preview() {
                     let _ = Root;
                     let scene = Scene { name: String::from(\"s\") };
                     let _ = run(&scene, &Config::default(), \"x\");
                 }
             }",
        )
        .unwrap();
        let previews = collect_previews(&file);
        assert_eq!(previews.len(), 1);
        assert!(previews[0].is_ignored);
        assert!(previews[0].is_routed_through_harness);
        assert!(previews[0].idents.iter().any(|ident| ident == "Root"));
    }

    #[test]
    fn test_usage_without_harness_is_not_routed() {
        let file = syn::parse_str(
            "#[cfg(test)]
             mod tests {
                 #[test]
                 fn wheel_ui_png_preview() {
                     let _ = run(&x, &y, \"z\");
                 }
             }",
        )
        .unwrap();
        let previews = collect_previews(&file);
        assert_eq!(previews.len(), 1);
        assert!(!previews[0].is_routed_through_harness);
    }
}
