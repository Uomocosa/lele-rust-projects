use syn::spanned::Spanned;
use syn::visit::Visit;

use lele_lint::diagnostic::Diagnostic;
use lele_lint::entry_kind::EntryKind;
use lele_lint::project::Project;
use lele_lint::severity::Severity;

use super::bevy_folder::BevyFolder;

const SYSTEM_PARAM_NAMES: [&str; 6] = [
    "Res",
    "ResMut",
    "Query",
    "Commands",
    "MessageWriter",
    "MessageReader",
];

pub(crate) fn check(_self: &BevyFolder, project: &Project) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    for (rel_path, file) in &project.parsed_files {
        if rel_path
            .components()
            .any(|c| c.as_os_str().to_str() == Some("bevy_systems"))
        {
            continue;
        }

        let registered = registered_systems(file);

        for item in &file.items {
            let syn::Item::Fn(func) = item else {
                continue;
            };

            if !matches!(func.vis, syn::Visibility::Public(_)) {
                continue;
            }

            if !func.sig.inputs.iter().any(is_bevy_system_param) {
                continue;
            }

            let ident = func.sig.ident.to_string();
            if !registered.contains(&ident) {
                continue;
            }

            let entry = project
                .entries
                .iter()
                .find(|e| e.relative_path == *rel_path && e.kind == EntryKind::File);

            diags.push(Diagnostic {
                file: entry
                    .map(|e| e.absolute_path.clone())
                    .unwrap_or_else(|| project.src_dir.join(rel_path)),
                line: func.sig.fn_token.span().start().line,
                col: 0,
                code: "E008".to_string(),
                message: format!(
                    "pub fn `{}` is registered with `app.add_systems()` but lives outside bevy_systems/; move it into the domain's bevy_systems/ folder",
                    func.sig.ident
                ),
                severity: Severity::Error,
            });
        }
    }

    diags
}

fn is_bevy_system_param(arg: &syn::FnArg) -> bool {
    let syn::FnArg::Typed(pat_type) = arg else {
        return false;
    };

    last_path_ident(&pat_type.ty).is_some_and(|ident| SYSTEM_PARAM_NAMES.contains(&ident.as_str()))
}

fn last_path_ident(ty: &syn::Type) -> Option<String> {
    match ty {
        syn::Type::Path(p) => p.path.segments.last().map(|s| s.ident.to_string()),
        syn::Type::Reference(r) => last_path_ident(&r.elem),
        _ => None,
    }
}

// needed helper: collects idents passed to `add_systems` after the schedule
fn registered_systems(file: &syn::File) -> Vec<String> {
    let mut collector = RegisteredSystems::default();
    collector.visit_file(file);
    collector.found
}

#[derive(Default)]
struct RegisteredSystems {
    armed: bool,
    found: Vec<String>,
}

impl<'ast> Visit<'ast> for RegisteredSystems {
    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        if node.method == "add_systems" {
            self.armed = true;
            let mut args = node.args.iter();
            args.next();
            for arg in args {
                self.visit_expr(arg);
            }
            self.armed = false;
        } else {
            syn::visit::visit_expr_method_call(self, node);
        }
    }

    fn visit_path(&mut self, node: &'ast syn::Path) {
        if self.armed {
            if let Some(last) = node.segments.last() {
                self.found.push(last.ident.to_string());
            }
        }
        syn::visit::visit_path(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::{is_bevy_system_param, last_path_ident, registered_systems};

    #[test]
    fn test_usage() {
        let file: syn::File = syn::parse_str(
            "pub fn tick(mut q: Query<&Foo>, cmds: Commands) {}
             pub fn helper(x: u32) {}",
        )
        .unwrap();

        let syn::Item::Fn(system_fn) = &file.items[0] else {
            panic!("expected fn");
        };
        let syn::Item::Fn(plain_fn) = &file.items[1] else {
            panic!("expected fn");
        };

        assert!(system_fn.sig.inputs.iter().any(is_bevy_system_param));
        assert!(!plain_fn.sig.inputs.iter().any(is_bevy_system_param));

        let ty: syn::Type = syn::parse_str("&mut Query<&Foo>").unwrap();
        assert_eq!(last_path_ident(&ty), Some("Query".to_string()));

        let scheduled: syn::File = syn::parse_str(
            "pub fn build(app: &mut App) {
                 app.add_systems(Update, tick);
                 app.add_systems(Startup, (setup, spawn).chain());
             }",
        )
        .unwrap();
        let registered = registered_systems(&scheduled);
        assert!(registered.contains(&"tick".to_string()));
        assert!(registered.contains(&"setup".to_string()));
        assert!(registered.contains(&"spawn".to_string()));
        assert!(!registered.contains(&"Update".to_string()));
        assert!(!registered.contains(&"Startup".to_string()));

        let empty: syn::File = syn::parse_str("pub fn helper(x: u32) {}").unwrap();
        assert_eq!(registered_systems(&empty).len(), 0);
    }
}
