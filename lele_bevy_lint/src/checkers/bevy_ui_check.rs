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

const PNG_SIGNALS: [&str; 2] = ["Screenshot", "save_to_disk"];

const MP4_SIGNALS: [&str; 5] = [
    "start_record_at",
    "drive_cursor",
    "place_window",
    "x11grab",
    "ffmpeg",
];

const CONTRACT_MARKER: &str = "PREVIEW_ARTIFACT=";

pub(crate) fn check(_self: &BevyUi, project: &Project) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    let graph = CallGraph::collect(project);

    for (rel_path, file) in &project.parsed_files {
        let stem = rel_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_string();
        let file_path = project
            .entries
            .iter()
            .find(|e| e.relative_path == *rel_path && e.kind == EntryKind::File)
            .map(|e| e.absolute_path.clone())
            .unwrap_or_else(|| project.src_dir.join(rel_path));

        let prod_visual = first_prod_visual(file);
        let previews = collect_previews(file);
        let (_png_signals, mp4_signals) = test_signals(file);

        if !file_reaches_production(file, &graph) {
            continue;
        }

        let reachable = prod_visual
            .as_ref()
            .is_some_and(|found| graph.reaches_production(&found.owner));
        let wants_png = reachable;
        let wants_mp4 = mp4_signals || previews.iter().any(|p| p.kind == PreviewKind::Mp4);

        if !wants_png && !wants_mp4 {
            continue;
        }

        let png_preview = previews.iter().find(|p| p.kind == PreviewKind::Png);
        let mp4_preview = previews.iter().find(|p| p.kind == PreviewKind::Mp4);

        if wants_png && png_preview.is_none() {
            let (visual, line) = prod_visual
                .as_ref()
                .map(|f| (f.visual.clone(), f.line))
                .unwrap_or_default();
            diags.push(Diagnostic {
                file: file_path.clone(),
                line,
                col: 0,
                code: "E029".to_string(),
                message: format!(
                    "file spawns Bevy visual component `{visual}` reachable from production code but defines no ignored `*_ui_png_preview` test ending with assert!(exists) + println!(PREVIEW_ARTIFACT=...); add one rendering this file's scene"
                ),
                severity: Severity::Error,
            });
        }

        if wants_mp4 && mp4_preview.is_none() {
            diags.push(Diagnostic {
                file: file_path.clone(),
                line: 1,
                col: 0,
                code: "E029".to_string(),
                message: "file drives a recording (start_record_at/drive_cursor/place_window) but defines no ignored `*_ui_mp4_preview` test ending with assert!(exists) + println!(PREVIEW_ARTIFACT=...)"
                    .to_string(),
                severity: Severity::Error,
            });
        }

        for preview in &previews {
            check_preview(preview, &stem, &file_path, &mut diags, file);
        }
    }

    diags
}

// needed helper: validates one preview test against the full contract
fn check_preview(
    preview: &Preview,
    stem: &str,
    file_path: &std::path::Path,
    diags: &mut Vec<Diagnostic>,
    file: &syn::File,
) {
    let ext = match preview.kind {
        PreviewKind::Png => "png",
        PreviewKind::Mp4 => "mp4",
    };
    if !preview.flags & FLAG_IGNORE != 0 {
        diags.push(Diagnostic {
            file: file_path.to_path_buf(),
            line: preview.line,
            col: 0,
            code: "E029".to_string(),
            message: format!(
                "`fn {}` must carry `#[ignore]` (headed preview)",
                preview.name
            ),
            severity: Severity::Error,
        });
    }
    if !preview.flags & FLAG_UNIT != 0 {
        diags.push(Diagnostic {
            file: file_path.to_path_buf(),
            line: preview.line,
            col: 0,
            code: "E029".to_string(),
            message: format!(
                "`fn {}` must return `()` (libtest only accepts `()` or `Result<(), E>`)",
                preview.name
            ),
            severity: Severity::Error,
        });
    }
    if !preview.flags & FLAG_ASSERT != 0 {
        diags.push(Diagnostic {
            file: file_path.to_path_buf(),
            line: preview.line,
            col: 0,
            code: "E029".to_string(),
            message: format!(
                "`fn {}` must keep a preceding `assert!(<artifact>.exists())` before the contract line",
                preview.name
            ),
            severity: Severity::Error,
        });
    }
    if !preview.flags & FLAG_CONTRACT != 0 {
        diags.push(Diagnostic {
            file: file_path.to_path_buf(),
            line: preview.line,
            col: 0,
            code: "E029".to_string(),
            message: format!(
                "`fn {}` must end with `println!(\"PREVIEW_ARTIFACT={{}}\", path.display())` as its last statement",
                preview.name
            ),
            severity: Severity::Error,
        });
    }
    if !shot_names_stem(file, stem, ext) {
        diags.push(Diagnostic {
            file: file_path.to_path_buf(),
            line: preview.line,
            col: 0,
            code: "E029".to_string(),
            message: format!(
                "`fn {}` does not reference `{stem}.{ext}`; name the capture file after the source file",
                preview.name
            ),
            severity: Severity::Error,
        });
    }
}

struct FoundVisual {
    visual: String,
    line: usize,
    owner: String,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PreviewKind {
    Png,
    Mp4,
}

struct Preview {
    name: String,
    kind: PreviewKind,
    line: usize,
    flags: u8,
}

const FLAG_IGNORE: u8 = 0b0001;
const FLAG_UNIT: u8 = 0b0010;
const FLAG_CONTRACT: u8 = 0b0100;
const FLAG_ASSERT: u8 = 0b1000;

struct UiScanner<'a> {
    found: &'a mut Vec<FoundVisual>,
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
        if node.method == "spawn" && self.found.is_empty() {
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
                self.found.push(FoundVisual {
                    visual: visual.clone(),
                    line: node.method.span().start().line,
                    owner: self.owner.clone(),
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

// needed helper: file scope gate via production reachability
fn file_reaches_production(file: &syn::File, graph: &CallGraph) -> bool {
    let mut names = Vec::new();
    {
        let mut collector = FnNameCollector(&mut names);
        for item in &file.items {
            if is_test_module(item) {
                continue;
            }
            collector.visit_item(item);
        }
    }
    names.iter().any(|name| graph.reaches_production(name))
}

// needed helper: collects non-test fn names for scope gating
#[derive(Deref, DerefMut)]
struct FnNameCollector<'a>(&'a mut Vec<String>);

impl<'ast> Visit<'ast> for FnNameCollector<'_> {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        self.push(node.sig.ident.to_string());
        syn::visit::visit_item_fn(self, node);
    }
}

// needed helper: first visual spawn in non-test items with its owner fn
fn first_prod_visual(file: &syn::File) -> Option<FoundVisual> {
    let mut found = Vec::new();
    let mut owner = String::new();
    {
        let mut scanner = UiScanner {
            found: &mut found,
            owner: &mut owner,
        };
        for item in &file.items {
            if is_test_module(item) {
                continue;
            }
            scanner.visit_item(item);
        }
    }
    found.into_iter().next()
}

// needed helper: call graph over non-test fns for reachability
struct CallGraph {
    callees: std::collections::HashMap<String, Vec<String>>,
    systems: std::collections::HashSet<String>,
}

impl CallGraph {
    fn collect(project: &Project) -> Self {
        let mut graph = Self {
            callees: std::collections::HashMap::new(),
            systems: std::collections::HashSet::new(),
        };
        for file in project.parsed_files.values() {
            for item in &file.items {
                if is_test_module(item) {
                    continue;
                }
                graph.visit_item(item);
            }
        }
        graph
    }

    // needed helper: production reachability from entry roots
    fn reaches_production(&self, owner: &str) -> bool {
        if owner.is_empty() {
            return true;
        }
        let mut roots = vec!["main".to_string(), "build".to_string(), "setup".to_string()];
        roots.extend(self.systems.iter().cloned());
        let mut seen = std::collections::HashSet::new();
        while let Some(current) = roots.pop() {
            if !seen.insert(current.clone()) {
                continue;
            }
            if current == owner {
                return true;
            }
            if let Some(next) = self.callees.get(&current) {
                roots.extend(next.iter().cloned());
            }
        }
        false
    }

    fn visit_item(&mut self, item: &syn::Item) {
        match item {
            syn::Item::Fn(func) => {
                let name = func.sig.ident.to_string();
                let mut calls = Vec::new();
                let mut systems = Vec::new();
                {
                    let mut collector = CallCollector {
                        calls: &mut calls,
                        systems: &mut systems,
                    };
                    collector.visit_block(&func.block);
                }
                self.callees.entry(name).or_default().extend(calls);
                self.systems.extend(systems);
            }
            syn::Item::Mod(module) => {
                if let Some((_, inner)) = &module.content {
                    for item in inner {
                        self.visit_item(item);
                    }
                }
            }
            _ => {}
        }
    }
}

struct CallCollector<'a> {
    calls: &'a mut Vec<String>,
    systems: &'a mut Vec<String>,
}

impl<'ast> Visit<'ast> for CallCollector<'_> {
    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if let syn::Expr::Path(path) = &*node.func {
            if let Some(last) = path.path.segments.last() {
                self.calls.push(last.ident.to_string());
            }
        }
        syn::visit::visit_expr_call(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        if node.method == "add_systems" {
            let mut args = node.args.iter();
            args.next();
            for arg in args {
                let mut collected = Vec::new();
                {
                    let mut paths = SystemPaths(&mut collected);
                    paths.visit_expr(arg);
                }
                self.calls.extend(collected.iter().cloned());
                self.systems.extend(collected);
            }
        } else {
            syn::visit::visit_expr_method_call(self, node);
        }
    }
}

// needed helper: path collector scoped to add_systems arguments
#[derive(Deref, DerefMut)]
struct SystemPaths<'a>(&'a mut Vec<String>);

impl<'ast> Visit<'ast> for SystemPaths<'_> {
    fn visit_path(&mut self, node: &'ast syn::Path) {
        if let Some(last) = node.segments.last() {
            self.0.push(last.ident.to_string());
        }
        syn::visit::visit_path(self, node);
    }
}

// needed helper: detects cfg(test) modules to skip test-only code
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

// needed helper: collects preview tests with contract attributes
fn collect_previews(file: &syn::File) -> Vec<Preview> {
    let mut previews = Vec::new();
    let mut finder = PreviewFinder(&mut previews);
    finder.visit_file(file);
    previews
}

#[derive(Deref, DerefMut)]
struct PreviewFinder<'a>(&'a mut Vec<Preview>);

impl<'ast> Visit<'ast> for PreviewFinder<'_> {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        let name = node.sig.ident.to_string();
        let kind = if name.ends_with("_ui_png_preview") {
            Some(PreviewKind::Png)
        } else if name.ends_with("_ui_mp4_preview") {
            Some(PreviewKind::Mp4)
        } else {
            None
        };
        if let Some(kind) = kind {
            let mut bits = 0;
            if node.attrs.iter().any(|attr| {
                attr.path()
                    .segments
                    .last()
                    .is_some_and(|s| s.ident == "ignore")
            }) {
                bits |= FLAG_IGNORE;
            }
            if matches!(node.sig.output, syn::ReturnType::Default) {
                bits |= FLAG_UNIT;
            }
            if last_is_contract(node) {
                bits |= FLAG_CONTRACT;
            }
            if body_has_assert_exists(node) {
                bits |= FLAG_ASSERT;
            }
            self.push(Preview {
                name,
                kind,
                line: node.sig.fn_token.span().start().line,
                flags: bits,
            });
        }
        syn::visit::visit_item_fn(self, node);
    }
}

// needed helper: verifies the println contract is the final statement
fn last_is_contract(func: &syn::ItemFn) -> bool {
    let Some(last) = func.block.stmts.last() else {
        return false;
    };
    match last {
        syn::Stmt::Macro(stmt_macro) => {
            is_println_contract(&stmt_macro.mac.path, &stmt_macro.mac.tokens.to_string())
        }
        syn::Stmt::Expr(expr, _) => match expr {
            syn::Expr::Macro(mac) => {
                is_println_contract(&mac.mac.path, &mac.mac.tokens.to_string())
            }
            _ => false,
        },
        syn::Stmt::Item(_) | syn::Stmt::Local(_) => false,
    }
}

// needed helper: println path plus marker matching
fn is_println_contract(path: &syn::Path, tokens: &str) -> bool {
    path.segments.last().is_some_and(|s| s.ident == "println") && tokens.contains(CONTRACT_MARKER)
}

// needed helper: requires a preceding assert exists guard
fn body_has_assert_exists(func: &syn::ItemFn) -> bool {
    let mut found = false;
    let mut checker = AssertExistsChecker(&mut found);
    checker.visit_block(&func.block);
    found
}

#[derive(Deref, DerefMut)]
struct AssertExistsChecker<'a>(&'a mut bool);

impl<'ast> Visit<'ast> for AssertExistsChecker<'_> {
    fn visit_expr_macro(&mut self, node: &'ast syn::ExprMacro) {
        if is_assert_exists(&node.mac.path, &node.mac.tokens.to_string()) {
            ***self = true;
        }
        syn::visit::visit_expr_macro(self, node);
    }

    fn visit_stmt_macro(&mut self, node: &'ast syn::StmtMacro) {
        if is_assert_exists(&node.mac.path, &node.mac.tokens.to_string()) {
            ***self = true;
        }
        syn::visit::visit_stmt_macro(self, node);
    }
}

// needed helper: assert with exists guard matching
fn is_assert_exists(path: &syn::Path, tokens: &str) -> bool {
    let compact: String = tokens.split_whitespace().collect();
    path.segments.last().is_some_and(|s| s.ident == "assert") && compact.contains(".exists()")
}

// needed helper: scans test code for recorder-kind signals
fn test_signals(file: &syn::File) -> (bool, bool) {
    let mut png = false;
    let mut mp4 = false;
    let mut collector = SignalCollector {
        png: &mut png,
        mp4: &mut mp4,
    };
    collector.visit_file(file);
    (png, mp4)
}

struct SignalCollector<'a> {
    png: &'a mut bool,
    mp4: &'a mut bool,
}

impl<'ast> Visit<'ast> for SignalCollector<'_> {
    fn visit_path(&mut self, node: &'ast syn::Path) {
        if let Some(last) = node.segments.last() {
            let name = last.ident.to_string();
            if PNG_SIGNALS.contains(&name.as_str()) {
                *self.png = true;
            }
            if MP4_SIGNALS.contains(&name.as_str()) {
                *self.mp4 = true;
            }
        }
        syn::visit::visit_path(self, node);
    }

    fn visit_lit_str(&mut self, node: &'ast syn::LitStr) {
        let value = node.value();
        if MP4_SIGNALS.iter().any(|s| value.contains(s)) {
            *self.mp4 = true;
        }
        syn::visit::visit_lit_str(self, node);
    }
}

// needed helper: stem-named artifact literal per extension
fn shot_names_stem(file: &syn::File, stem: &str, ext: &str) -> bool {
    let mut finder = StemFinder {
        stem,
        ext,
        found: false,
    };
    finder.visit_file(file);
    finder.found
}

struct StemFinder<'a> {
    stem: &'a str,
    ext: &'a str,
    found: bool,
}

impl Visit<'_> for StemFinder<'_> {
    fn visit_lit_str(&mut self, node: &syn::LitStr) {
        if node.value() == format!("{}.{}", self.stem, self.ext) {
            self.found = true;
        }
        syn::visit::visit_lit_str(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        collect_previews, shot_names_stem, test_signals, FLAG_ASSERT, FLAG_CONTRACT, FLAG_IGNORE,
        FLAG_UNIT,
    };

    fn parse(source: &str) -> syn::File {
        syn::parse_str(source).unwrap()
    }

    #[test]
    fn test_usage() {
        let with_preview = parse(
            "pub struct Spawner;
             pub fn spawn_target(s: &mut Spawner) { s.spawn((Sprite,)); }
             #[cfg(test)] mod tests {
                 #[test] #[ignore = \"headed\"] fn target_ui_png_preview() {
                     let p = std::path::PathBuf::from(\"x\").join(\"target.png\");
                     assert!(p.exists());
                     println!(\"PREVIEW_ARTIFACT={}\", p.display());
                 }
             }",
        );
        let previews = collect_previews(&with_preview);
        assert_eq!(previews.len(), 1);
        assert!(previews[0].flags & FLAG_IGNORE != 0);
        assert!(previews[0].flags & FLAG_UNIT != 0);
        assert!(previews[0].flags & FLAG_CONTRACT != 0);
        assert!(previews[0].flags & FLAG_ASSERT != 0);
        assert!(shot_names_stem(&with_preview, "target", "png"));
        assert!(!shot_names_stem(&with_preview, "other", "png"));
        let (png, _) = test_signals(&with_preview);
        assert!(!png);

        let missing_contract = parse(
            "#[cfg(test)] mod tests {
                 #[test] #[ignore = \"h\"] fn target_ui_png_preview() {
                     let p = std::path::PathBuf::from(\"x\");
                     assert!(p.exists());
                 }
             }",
        );
        let previews = collect_previews(&missing_contract);
        assert_eq!(previews.len(), 1);
        assert!(!previews[0].flags & FLAG_CONTRACT != 0);
        assert!(previews[0].flags & FLAG_ASSERT != 0);

        let wrong_suffix = parse(
            "#[cfg(test)] mod tests {
                 #[test] #[ignore = \"h\"] fn ui_png() {}
             }",
        );
        assert!(collect_previews(&wrong_suffix).is_empty());

        let mp4_signals = parse(
            "#[cfg(test)] mod tests {
                 #[test] #[ignore = \"h\"] fn target_ui_mp4_preview() {
                     drive_cursor(\"t\");
                     assert!(p.exists());
                     println!(\"PREVIEW_ARTIFACT={}\", p.display());
                 }
             }",
        );
        let (_, mp4) = test_signals(&mp4_signals);
        assert!(mp4);
    }

    #[test]
    fn test_usage_unignored_preview_fails_contract() {
        let file = parse(
            "#[cfg(test)] mod tests {
                 #[test] fn target_ui_png_preview() {
                     println!(\"PREVIEW_ARTIFACT={}\", p.display());
                 }
             }",
        );
        let previews = collect_previews(&file);
        assert_eq!(previews.len(), 1);
        assert!(!previews[0].flags & FLAG_IGNORE != 0);
        assert!(previews[0].flags & FLAG_CONTRACT != 0);
        assert!(!previews[0].flags & FLAG_ASSERT != 0);
    }

    #[test]
    fn test_usage_non_unit_return_rejected() {
        let file = parse(
            "#[cfg(test)] mod tests {
                 #[test] #[ignore = \"h\"] fn target_ui_png_preview() -> u32 {
                     println!(\"PREVIEW_ARTIFACT={}\", p.display());
                     1
                 }
             }",
        );
        let previews = collect_previews(&file);
        assert_eq!(previews.len(), 1);
        assert!(!previews[0].flags & FLAG_UNIT != 0);
    }
}
