use std::path::PathBuf;

pub fn run(manifest_path: Option<PathBuf>, honesty_depth: Option<usize>) -> i32 {
    let mut ctx = crate::config::load(manifest_path.as_deref());
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

    use crate::call_graph::CallGraph;
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

        let diags = crate::depth_check::check_depth(&graph, &taxonomy, &file_of, &ctx);
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
        taxonomy: &mut HashMap<String, crate::taxonomy::FunctionTaxonomy>,
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
        taxonomy: &mut HashMap<String, crate::taxonomy::FunctionTaxonomy>,
        file_of: &mut HashMap<String, PathBuf>,
    ) {
        use syn::visit::Visit;
        struct FnVisitor<'a> {
            #[allow(dead_code)]
            ctx: &'a TaxonomyCtx,
            fns: Vec<(String, bool, Vec<String>, bool)>,
            current_callees: Vec<String>,
            current_is_const: bool,
            current_has_hidden: bool,
        }
        impl<'ast> Visit<'ast> for FnVisitor<'_> {
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
                self.current_is_const = is_const;
                self.current_has_hidden = has_hidden;
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
            ctx,
            fns: Vec::new(),
            current_callees: Vec::new(),
            current_is_const: false,
            current_has_hidden: false,
        };
        v.visit_file(file);
        let rel = path.strip_prefix(root).unwrap_or(path).to_path_buf();
        for (name, is_const, callees, has_hidden) in v.fns {
            let has_hidden_write =
                has_hidden && quote::quote!(#file).to_string().contains("static");
            let r = crate::hir_visitor::analyze_stub(
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
