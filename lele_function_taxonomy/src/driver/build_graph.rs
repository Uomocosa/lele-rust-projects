use std::collections::HashMap;
use std::path::{Path, PathBuf};

use rustc_errors::DiagCtxtHandle;
use rustc_hir::def::DefKind;
use rustc_hir::def_id::LocalDefId;
use rustc_middle::mir::{Body, Operand, Rvalue, StatementKind, TerminatorKind};
use rustc_middle::ty::{self, Instance, TyCtxt};
use rustc_span::Span;

use lele_function_taxonomy::honesty_graph;

use crate::roots;

pub struct HonestBoundary {
    pub name: String,
    pub folders: Vec<String>,
}

pub struct ReportConfig {
    pub honest_boundaries: Vec<HonestBoundary>,
    pub declared_dishonest: Vec<String>,
    pub declared_honest: Vec<String>,
}

impl ReportConfig {
    // needed helper: the boundary describing a crate-relative file, if any
    pub fn boundary_for(&self, rel: &Path) -> Option<&str> {
        self.honest_boundaries
            .iter()
            .find(|b| b.folders.iter().any(|f| rel.starts_with(f)))
            .map(|b| b.name.as_str())
    }
}

pub fn run(tcx: TyCtxt<'_>) {
    let config = super::read_config();
    let crate_root = crate_root_dir();
    let built = build(tcx, &config);

    let witnesses = built.graph.classify();

    let mut diagnostics: Vec<String> = Vec::new();

    let mut ordered: Vec<(String, LocalDefId)> = built
        .paths
        .iter()
        .map(|(id, path)| (path.clone(), *id))
        .collect();
    ordered.sort_by(|a, b| a.0.cmp(&b.0));

    for (path_str, def_id) in ordered {
        let Some(span) = built.spans.get(&def_id).copied() else {
            continue;
        };
        let Some(rel) = span_file(tcx, span, &crate_root) else {
            continue;
        };
        let Some(boundary) = config.boundary_for(&rel) else {
            continue;
        };
        let node = built.node_ids.get(&def_id).copied();
        let Some(node) = node else {
            continue;
        };
        let Some(witness) = witnesses.get(&node) else {
            continue;
        };
        let location = format!(
            "{}:{}:{}",
            rel.display(),
            line_of(tcx, span),
            col_of(tcx, span)
        );
        diagnostics.push(render(&location, &path_str, witness, &built, boundary));
    }

    if diagnostics.is_empty() {
        return;
    }
    let dcx: DiagCtxtHandle<'_> = tcx.dcx();
    for message in diagnostics {
        dcx.struct_err(message).emit();
    }
}

struct Built {
    graph: honesty_graph::HonestyGraph,
    paths: HashMap<LocalDefId, String>,
    node_ids: HashMap<LocalDefId, honesty_graph::NodeId>,
    spans: HashMap<LocalDefId, Span>,
}

fn build(tcx: TyCtxt<'_>, config: &ReportConfig) -> Built {
    let mut graph = honesty_graph::HonestyGraph::default();
    let mut paths: HashMap<LocalDefId, String> = HashMap::new();
    let mut node_ids: HashMap<LocalDefId, honesty_graph::NodeId> = HashMap::new();
    let mut spans: HashMap<LocalDefId, Span> = HashMap::new();

    let mut owners: Vec<(String, LocalDefId)> = tcx
        .hir_body_owners()
        .filter(|d| is_function_like(tcx, *d))
        .map(|d| (tcx.def_path_str(d.to_def_id()), d))
        .collect();
    owners.sort_by(|a, b| a.0.cmp(&b.0));

    for (index, (path_str, def_id)) in owners.iter().enumerate() {
        let id = honesty_graph::NodeId(u32::try_from(index).unwrap_or(u32::MAX));
        node_ids.insert(*def_id, id);
        paths.insert(*def_id, path_str.clone());
        if let Some(span) = tcx.def_ident_span(def_id.to_def_id()) {
            spans.insert(*def_id, span);
        }
        let direct = body_cause(tcx, *def_id, config);
        graph.add_node(id, path_str.clone(), direct);
    }

    for (_, def_id) in &owners {
        let Some(caller) = node_ids.get(def_id).copied() else {
            continue;
        };
        for callee_def in body_calls(tcx, *def_id, &node_ids) {
            if let Some(callee) = node_ids.get(&callee_def).copied() {
                graph.add_edge(caller, callee);
            }
        }
        for child in closures_of(tcx, *def_id) {
            if let Some(target) = node_ids.get(&child).copied() {
                graph.add_edge(caller, target);
            }
        }
    }

    Built {
        graph,
        paths,
        node_ids,
        spans,
    }
}

fn is_function_like(tcx: TyCtxt<'_>, def_id: LocalDefId) -> bool {
    matches!(
        tcx.def_kind(def_id),
        DefKind::Fn | DefKind::AssocFn | DefKind::Closure | DefKind::SyntheticCoroutineBody
    )
}

// needed helper: direct dishonesty of a single body (statics, thread-locals, I/O-root calls)
fn body_cause(
    tcx: TyCtxt<'_>,
    def_id: LocalDefId,
    config: &ReportConfig,
) -> Option<honesty_graph::DirectCause> {
    use honesty_graph::DirectCause;
    let path_str = tcx.def_path_str(def_id.to_def_id());

    if matches_declared(&path_str, &config.declared_honest) {
        return None;
    }
    if matches_declared(&path_str, &config.declared_dishonest) {
        return Some(DirectCause::DeclaredDishonest(path_str));
    }

    let body = tcx.optimized_mir(def_id);
    let env = body.typing_env(tcx);

    for bb in body.basic_blocks.iter() {
        for stmt in &bb.statements {
            if let StatementKind::Assign(assign) = &stmt.kind {
                if let Some(cause) = static_cause(tcx, env, &assign.1, &config.declared_honest) {
                    return Some(cause);
                }
            }
        }
        let Some(term) = &bb.terminator else {
            continue;
        };
        match &term.kind {
            TerminatorKind::Call { func, .. } | TerminatorKind::TailCall { func, .. } => {
                if let Some(cause) = call_cause(tcx, body, func, &config.declared_honest) {
                    return Some(cause);
                }
            }
            _ => {}
        }
    }
    None
}

fn matches_declared(path: &str, list: &[String]) -> bool {
    list.iter()
        .any(|d| path == *d || path.ends_with(d.as_str()))
}

fn static_cause<'tcx>(
    tcx: TyCtxt<'tcx>,
    env: ty::TypingEnv<'tcx>,
    rvalue: &Rvalue<'tcx>,
    declared_honest: &[String],
) -> Option<honesty_graph::DirectCause> {
    use honesty_graph::DirectCause;
    match rvalue {
        Rvalue::Use(op, _) | Rvalue::Repeat(op, _) | Rvalue::Cast(_, op, _) => {
            let Operand::Constant(constant) = op else {
                return None;
            };
            let static_id = constant.check_static_ptr(tcx)?;
            if matches_declared(&tcx.def_path_str(static_id), declared_honest) {
                return None;
            }
            let name = tcx.def_path_str(static_id);
            if tcx.is_mutable_static(static_id) {
                return Some(DirectCause::HiddenStaticWrite(name));
            }
            let ty = tcx
                .type_of(static_id)
                .instantiate_identity()
                .skip_normalization();
            if ty.is_freeze(tcx, env) {
                return None;
            }
            if is_logging_static(tcx, ty) {
                return None;
            }
            Some(DirectCause::HiddenStaticRead(name))
        }
        Rvalue::ThreadLocalRef(def_id) => Some(DirectCause::ThreadLocal(tcx.def_path_str(*def_id))),
        _ => None,
    }
}

// needed helper: tracing/log callsite statics are sanctioned (logging is honest everywhere)
fn is_logging_static<'tcx>(tcx: TyCtxt<'tcx>, ty: ty::Ty<'tcx>) -> bool {
    match ty.kind() {
        ty::Adt(adt, _) => roots::is_logging_type(&tcx.def_path_str(adt.did())),
        ty::Ref(_, inner, _) | ty::RawPtr(inner, _) => is_logging_static(tcx, *inner),
        ty::Array(inner, _) | ty::Slice(inner) => is_logging_static(tcx, *inner),
        _ => false,
    }
}

fn call_cause<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &Body<'tcx>,
    func: &Operand<'tcx>,
    declared_honest: &[String],
) -> Option<honesty_graph::DirectCause> {
    use honesty_graph::DirectCause;
    let ty = func.ty(body, tcx);
    let ty::FnDef(def_id, args) = ty.kind() else {
        return None;
    };
    let env = body.typing_env(tcx);
    let callee = match Instance::try_resolve(tcx, env, *def_id, args.skip_binder()) {
        Ok(Some(instance)) => instance.def_id(),
        _ => *def_id,
    };
    let path = tcx.def_path_str(callee);
    if matches_declared(&path, declared_honest) {
        return None;
    }
    roots::is_io_root(&path, &[]).map(DirectCause::IoRoot)
}

// needed helper: resolved callee DefIds of a body, keyed by the local map
fn body_calls(
    tcx: TyCtxt<'_>,
    def_id: LocalDefId,
    local: &HashMap<LocalDefId, honesty_graph::NodeId>,
) -> Vec<LocalDefId> {
    let body = tcx.optimized_mir(def_id);
    let env = body.typing_env(tcx);
    let mut out = Vec::new();
    for bb in body.basic_blocks.iter() {
        let Some(term) = &bb.terminator else {
            continue;
        };
        if let TerminatorKind::Call { func, .. } | TerminatorKind::TailCall { func, .. } =
            &term.kind
        {
            let ty = func.ty(body, tcx);
            if let ty::FnDef(callee, args) = ty.kind() {
                let resolved = match Instance::try_resolve(tcx, env, *callee, args.skip_binder()) {
                    Ok(Some(instance)) => instance.def_id(),
                    _ => *callee,
                };
                if let Some(local_id) = resolved.as_local() {
                    if local.contains_key(&local_id) {
                        out.push(local_id);
                    }
                }
            }
        }
    }
    out
}

// needed helper: closures/coroutines physically defined inside a body
fn closures_of(tcx: TyCtxt<'_>, def_id: LocalDefId) -> Vec<LocalDefId> {
    let mut out = Vec::new();
    for child in tcx.hir_body_owners() {
        if !matches!(
            tcx.def_kind(child),
            DefKind::Closure | DefKind::SyntheticCoroutineBody
        ) {
            continue;
        }
        if tcx.local_parent(child) == def_id {
            out.push(child);
        }
    }
    out
}

// needed helper: crate-relative source path of a span
fn span_file(tcx: TyCtxt<'_>, span: Span, crate_root: &Path) -> Option<PathBuf> {
    let source_map = tcx.sess.source_map();
    let file = source_map.lookup_source_file(span.source_callsite().lo());
    let path = file.name.clone().into_local_path()?;
    let rel = path.strip_prefix(crate_root).unwrap_or(&path);
    Some(rel.to_path_buf())
}

fn crate_root_dir() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

fn render(
    location: &str,
    path_str: &str,
    witness: &honesty_graph::Witness,
    built: &Built,
    boundary: &str,
) -> String {
    let chain: Vec<String> = witness
        .chain
        .iter()
        .map(|id| built.graph.path_of(*id).to_string())
        .collect();
    let hops = chain.join(" -> ");
    format!(
        "{location}: error[TAX001]: `{path_str}` is dishonest: {hops} ({}); boundary \"{boundary}\" requires honest functions",
        witness.cause
    )
}

fn line_of(tcx: TyCtxt<'_>, span: Span) -> usize {
    let loc = tcx.sess.source_map().lookup_char_pos(span.lo());
    loc.line
}

fn col_of(tcx: TyCtxt<'_>, span: Span) -> usize {
    let loc = tcx.sess.source_map().lookup_char_pos(span.lo());
    loc.col_display.saturating_add(1)
}

// no test_usage necessary
