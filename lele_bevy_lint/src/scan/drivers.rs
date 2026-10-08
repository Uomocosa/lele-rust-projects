use derive_more::{Deref, DerefMut};
use syn::visit::Visit;

use crate::scan;

const DRIVER_TOKENS: [&str; 16] = [
    "delta_secs",
    "elapsed_secs",
    "delta",
    "Timer",
    "Time",
    "Animatable",
    "AnimatableUi",
    "AnimationClip",
    "AnimationPlayer",
    "tween",
    "keyframe",
    "keyframes",
    "Interaction",
    "ButtonInput",
    "MouseButton",
    "KeyCode",
];

#[must_use]
pub fn drivers(file: &syn::File) -> Vec<String> {
    let mut found = Vec::new();
    {
        let mut scanner = DriverScanner(&mut found);
        for item in &file.items {
            if scan::is_test_module(item) {
                continue;
            }
            scanner.visit_item(item);
        }
    }
    found.sort();
    found.dedup();
    found
}

// needed helper: matches time and input driver tokens in paths and method calls
#[derive(Deref, DerefMut)]
struct DriverScanner<'a>(&'a mut Vec<String>);

impl<'ast> Visit<'ast> for DriverScanner<'_> {
    fn visit_path(&mut self, node: &'ast syn::Path) {
        for segment in &node.segments {
            let name = segment.ident.to_string();
            if DRIVER_TOKENS.contains(&name.as_str()) {
                self.push(name);
            }
        }
        syn::visit::visit_path(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        let name = node.method.to_string();
        if DRIVER_TOKENS.contains(&name.as_str()) {
            self.push(name);
        }
        syn::visit::visit_expr_method_call(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::drivers;

    #[test]
    fn test_usage() {
        let file = syn::parse_str(
            "pub fn tick(time: Res<Time>, q: Query<&Interaction>) {
                 let d = time.delta_secs();
                 if q.is_changed() { let _ = d; }
             }",
        )
        .unwrap();
        let found = drivers(&file);
        assert!(found.contains(&"Time".to_string()));
        assert!(found.contains(&"delta_secs".to_string()));
        assert!(found.contains(&"Interaction".to_string()));
        assert!(!found.contains(&"is_changed".to_string()));
    }

    #[test]
    fn test_usage_change_detection_is_not_a_driver() {
        let file = syn::parse_str(
            "pub fn sync(snapshot: Res<Snapshot>, mut ready: Local<bool>, q: Query<Entity, Changed<Name>>) {
                 if snapshot.is_changed() { *ready = true; }
             }",
        )
        .unwrap();
        assert_eq!(drivers(&file).len(), 0);
    }

    #[test]
    fn test_usage_plain_spawn_has_no_driver() {
        let file = syn::parse_str("pub fn setup(s: &mut S) { s.spawn((Node,)); }").unwrap();
        assert_eq!(drivers(&file).len(), 0);
    }
}
