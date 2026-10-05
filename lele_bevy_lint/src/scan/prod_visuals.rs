use derive_more::{Deref, DerefMut};
use syn::visit::Visit;

use crate::scan;

const VISUAL_IDENTS: [&str; 7] = [
    "Sprite",
    "Text2d",
    "Text",
    "Mesh2d",
    "MeshMaterial2d",
    "Node",
    "ImageNode",
];

#[must_use]
pub fn prod_visuals(file: &syn::File) -> Vec<scan::FoundVisual> {
    let mut found = Vec::new();
    let mut owner = String::new();
    {
        let mut scanner = UiScanner {
            found: &mut found,
            owner: &mut owner,
        };
        for item in &file.items {
            if scan::is_test_module(item) {
                continue;
            }
            scanner.visit_item(item);
        }
    }
    found
}

// needed helper: collects every non-test spawn bundle containing a visual ident
struct UiScanner<'a> {
    found: &'a mut Vec<scan::FoundVisual>,
    owner: &'a mut String,
}

impl<'ast> Visit<'ast> for UiScanner<'_> {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        let outer = std::mem::take(self.owner);
        *self.owner = node.sig.ident.to_string();
        syn::visit::visit_item_fn(self, node);
        *self.owner = outer;
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        if node.method == "spawn" {
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
                self.found.push(scan::FoundVisual {
                    visual: visual.clone(),
                    line: node.method.span().start().line,
                    owner: self.owner.clone(),
                });
            }
        }
        syn::visit::visit_expr_method_call(self, node);
    }
}

// needed helper: every path ident inside a spawn argument
#[derive(Deref, DerefMut)]
struct PathNames<'a>(&'a mut Vec<String>);

impl<'ast> Visit<'ast> for PathNames<'_> {
    fn visit_path(&mut self, node: &'ast syn::Path) {
        for segment in &node.segments {
            self.0.push(segment.ident.to_string());
        }
        syn::visit::visit_path(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::prod_visuals;

    #[test]
    fn test_usage() {
        let file = syn::parse_str(
            "pub struct Spawner;
             pub fn setup(spawner: &mut Spawner) { spawner.spawn((Node, Text::new(\"x\"))); }
             #[cfg(test)]
             mod tests { fn test_usage() { spawner.spawn((Node,)); } }",
        )
        .unwrap();
        let found = prod_visuals(&file);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].visual, "Node");
        assert_eq!(found[0].owner, "setup");
    }

    #[test]
    fn test_usage_camera_only_is_not_a_visual() {
        let file = syn::parse_str("pub fn setup(c: &mut S) { c.spawn((Camera2d,)); }").unwrap();
        assert_eq!(prod_visuals(&file).len(), 0);
    }
}
