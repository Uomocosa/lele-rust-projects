use syn::spanned::Spanned;

use super::mod_rs_purity::ModRsPurity;
use crate::common;
use crate::Diagnostic;
use crate::Project;
use crate::Severity;

pub(crate) fn check(_self: &ModRsPurity, project: &Project) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    for (rel_path, file) in &project.parsed_files {
        let Some(file_name) = rel_path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if file_name != "mod.rs" {
            continue;
        }

        for item in &file.items {
            let Some(message) = violation_message(item) else {
                continue;
            };
            diags.push(Diagnostic {
                file: project.src_dir.join(rel_path),
                line: item.span().start().line,
                col: 0,
                code: "E019".to_string(),
                message,
                severity: Severity::Error,
            });
        }
    }

    diags
}

// needed helper: mod.rs item violation classification
fn violation_message(item: &syn::Item) -> Option<String> {
    match item {
        syn::Item::Mod(module) if common::is_cfg_test_mod(module) => Some(
            "mod.rs may contain only `mod`/`pub mod` declarations and `pub use` re-exports — `#[cfg(test)] mod` test modules are not allowed"
                .to_string(),
        ),
        syn::Item::Mod(_) => None,
        syn::Item::Use(item_use) => {
            if matches!(item_use.vis, syn::Visibility::Inherited) {
                Some(
                    "mod.rs may contain only `pub use` re-exports — private `use` imports are not allowed; re-export with `pub use` or move the import into a function file"
                        .to_string(),
                )
            } else {
                None
            }
        }
        syn::Item::Struct(s) => Some(named_message("struct", &s.ident.to_string(), &s.vis)),
        syn::Item::Enum(e) => Some(named_message("enum", &e.ident.to_string(), &e.vis)),
        syn::Item::Fn(f) => Some(named_message("fn", &f.sig.ident.to_string(), &f.vis)),
        syn::Item::Const(c) => Some(named_message("const", &c.ident.to_string(), &c.vis)),
        syn::Item::Static(s) => Some(named_message("static", &s.ident.to_string(), &s.vis)),
        syn::Item::Trait(t) => Some(named_message("trait", &t.ident.to_string(), &t.vis)),
        syn::Item::Type(t) => Some(named_message("type alias", &t.ident.to_string(), &t.vis)),
        syn::Item::Impl(_) => Some(
            "mod.rs may contain only `mod`/`pub mod` declarations and `pub use` re-exports — `impl` blocks are not allowed"
                .to_string(),
        ),
        syn::Item::Macro(_) => Some(
            "mod.rs may contain only `mod`/`pub mod` declarations and `pub use` re-exports — macro definitions are not allowed"
                .to_string(),
        ),
        _ => Some(
            "mod.rs may contain only `mod`/`pub mod` declarations and `pub use` re-exports — move this item to its own file"
                .to_string(),
        ),
    }
}

// needed helper: named item violation message
fn named_message(kind: &str, name: &str, vis: &syn::Visibility) -> String {
    let vis_str = match vis {
        syn::Visibility::Public(_) => "pub ",
        syn::Visibility::Restricted(_) => "pub(crate) ",
        syn::Visibility::Inherited => "",
    };
    format!(
        "mod.rs may contain only `mod`/`pub mod` declarations and `pub use` re-exports — found `{vis_str}{kind} {name}`; move it to its own file"
    )
}

#[cfg(test)]
mod tests {
    use super::violation_message;
    use syn::Item;

    fn message(code: &str) -> Option<String> {
        let item: Item = syn::parse_str(code).unwrap();
        violation_message(&item)
    }

    #[test]
    fn test_usage() {
        assert!(message("mod player;").is_none());
        assert!(message("pub use player::Player;").is_none());
        assert!(message("use crate::checker;").is_some());
    }

    #[test]
    fn test_usage_allows_mod_declarations() {
        assert!(message("mod player;").is_none());
        assert!(message("pub mod bevy_systems;").is_none());
        assert!(message("pub(crate) mod spawned_peer;").is_none());
    }

    #[test]
    fn test_usage_allows_pub_use_reexports() {
        assert!(message("pub use player::Player;").is_none());
        assert!(message("pub(crate) use spawned_peer::SpawnedPeer;").is_none());
        assert!(message("pub use constants::*;").is_none());
    }

    #[test]
    fn test_usage_flags_private_use() {
        let m = message("use crate::checker;").unwrap();
        assert!(m.contains("private `use` import"));
    }

    #[test]
    fn test_usage_flags_test_module() {
        let m = message("#[cfg(test)] mod tests { fn test_usage() {} }").unwrap();
        assert!(m.contains("#[cfg(test)] mod"));
    }

    #[test]
    fn test_usage_flags_logic_items() {
        assert!(message("pub struct Score { pub value: u32 }")
            .unwrap()
            .contains("struct"));
        assert!(message("pub fn build_checkers() {}")
            .unwrap()
            .contains("fn build_checkers"));
        assert!(message("impl Player { fn tick(&self) {} }")
            .unwrap()
            .contains("impl"));
        assert!(message("pub const MAX: u32 = 10;")
            .unwrap()
            .contains("const"));
    }
}
