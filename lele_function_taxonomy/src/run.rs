use std::path::PathBuf;

use crate::load;

pub fn run(manifest_path: Option<PathBuf>, honesty_depth: Option<usize>) -> i32 {
    let mut ctx = load::load(manifest_path.as_deref());
    if let Some(d) = honesty_depth {
        ctx.honesty_depth = d;
    }
    ctx.manifest_path = manifest_path.clone();

    if let Some(manifest) = &ctx.manifest_path {
        if let Ok(meta) = cargo_metadata::MetadataCommand::new()
            .manifest_path(manifest)
            .exec()
        {
            tracing::info!(root = %meta.workspace_root, "loaded cargo metadata");
        }
    }

    #[cfg(feature = "rustc-private")]
    {
        if let Some(manifest) = manifest_path {
            return rustc_private_driver::run_with_rustc(manifest, ctx);
        }
        tracing::info!(
            honesty_depth = ctx.honesty_depth,
            entry_allowlist = ?ctx.entry_allowlist,
            "no manifest_path — skipping rustc walk"
        );
        0
    }

    #[cfg(not(feature = "rustc-private"))]
    {
        tracing::info!(
            honesty_depth = ctx.honesty_depth,
            entry_allowlist = ?ctx.entry_allowlist,
            "lele_function_taxonomy driver (stub — build with --features rustc-private on nightly for TyCtxt walk)"
        );
        let _ = manifest_path;
        0
    }
}

#[cfg(feature = "rustc-private")]
pub mod rustc_private_driver {
    use std::collections::HashMap;
    use std::path::PathBuf;

    use crate::analyze_stub;
    use crate::call_graph::CallGraph;
    use crate::check_depth;
    use crate::diagnostic::Diagnostic;
    use crate::function_taxonomy::FunctionTaxonomy;
    use crate::hir_honesty_result::HirHonestyResult;
    use crate::taxonomy_ctx::TaxonomyCtx;

    pub fn run_with_rustc(manifest: PathBuf, ctx: TaxonomyCtx) -> i32 {
        tracing::info!(
            manifest = %manifest.display(),
            honesty_depth = ctx.honesty_depth,
            entry_allowlist = ?ctx.entry_allowlist,
            "rustc-private driver: running syn-backed walk (TyCtxt wiring deferred to nightly API stabilization)"
        );

        let root = manifest.parent().unwrap_or(std::path::Path::new("."));
        let src_dir = root.join("src");
        let search_roots = if src_dir.exists() {
            vec![src_dir]
        } else {
            vec![root.to_path_buf()]
        };

        let mut graph = CallGraph::default();
        let mut taxonomy = HashMap::new();
        let mut file_of = HashMap::new();

        for search_root in search_roots {
            walk_dir_syn(
                &search_root,
                &search_root,
                &ctx,
                &mut graph,
                &mut taxonomy,
                &mut file_of,
            );
        }

        let diags: Vec<Diagnostic> = check_depth::check_depth(&graph, &taxonomy, &file_of, &ctx);
        if diags.is_empty() {
            tracing::info!("honesty check passed — no E023 violations");
            0
        } else {
            for d in &diags {
                eprintln!(
                    "{}:{}:{}: {} {}",
                    d.file.display(),
                    d.line,
                    d.col,
                    d.code,
                    d.message
                );
            }
            tracing::warn!(count = diags.len(), "honesty violations found");
            1
        }
    }

    fn walk_dir_syn(
        dir: &std::path::Path,
        root: &std::path::Path,
        ctx: &TaxonomyCtx,
        graph: &mut CallGraph,
        taxonomy: &mut HashMap<String, FunctionTaxonomy>,
        file_of: &mut HashMap<String, PathBuf>,
    ) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.is_dir() {
                walk_dir_syn(&path, root, ctx, graph, taxonomy, file_of);
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(file) = syn::parse_file(&content) {
                        analyze_file_syn(&file, &path, root, ctx, graph, taxonomy, file_of);
                    }
                }
            }
        }
    }

    fn analyze_file_syn(
        file: &syn::File,
        path: &std::path::Path,
        root: &std::path::Path,
        ctx: &TaxonomyCtx,
        graph: &mut CallGraph,
        taxonomy: &mut HashMap<String, FunctionTaxonomy>,
        file_of: &mut HashMap<String, PathBuf>,
    ) {
        use syn::visit::Visit;
        struct FnVisitor {
            fns: Vec<(String, bool, Vec<String>, bool)>,
            current_callees: Vec<String>,
        }
        impl<'ast> Visit<'ast> for FnVisitor {
            fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
                let name = node.sig.ident.to_string();
                let is_const = node.sig.constness.is_some();
                let content = quote::quote!(#node).to_string();
                let has_hidden = content.contains("SystemTime :: now")
                    || content.contains("Instant :: now")
                    || content.contains("thread_rng")
                    || content.contains("getrandom")
                    || content.contains("std :: env ::")
                    || content.contains("std :: fs ::")
                    || content.contains("static ");
                let mut callees = Vec::new();
                let prev = std::mem::take(&mut self.current_callees);
                self.current_callees.clear();
                syn::visit::visit_item_fn(self, node);
                callees.clone_from(&self.current_callees);
                self.fns.push((name, is_const, callees, has_hidden));
                self.current_callees = prev;
            }
            fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
                let s = quote::quote!(#node.func).to_string().replace(' ', "");
                if !s.is_empty() {
                    self.current_callees.push(s);
                }
                syn::visit::visit_expr_call(self, node);
            }
        }
        let mut v = FnVisitor {
            fns: Vec::new(),
            current_callees: Vec::new(),
        };
        v.visit_file(file);
        let rel = path.strip_prefix(root).unwrap_or(path).to_path_buf();
        for (name, is_const, callees, has_hidden) in v.fns {
            let has_hidden_write =
                has_hidden && quote::quote!(#file).to_string().contains("static");
            let r: HirHonestyResult = analyze_stub::analyze_stub(
                &name,
                ctx,
                has_hidden,
                has_hidden_write,
                is_const,
                callees.clone(),
                taxonomy,
            );
            let key = name.clone();
            for callee in &callees {
                let callee_simple = callee.rsplit("::").next().unwrap_or(callee).to_string();
                graph.add_edge(key.clone(), callee_simple);
            }
            taxonomy.insert(key.clone(), r.taxonomy);
            file_of.insert(key, rel.clone());
        }
    }
}

// no test_usage necessary
