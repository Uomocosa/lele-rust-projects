use std::collections::HashMap;
use std::collections::VecDeque;
use std::path::Path;

use crate::call_graph::CallGraph;
use crate::diagnostic::Diagnostic;
use crate::taxonomy::DishonestReason;
use crate::taxonomy::FunctionTaxonomy;
use crate::taxonomy_ctx::TaxonomyCtx;

pub fn check_depth(
    graph: &CallGraph,
    taxonomy: &HashMap<String, FunctionTaxonomy>,
    file_of: &HashMap<String, std::path::PathBuf, std::collections::hash_map::RandomState>,
    ctx: &TaxonomyCtx,
) -> Vec<Diagnostic> {
    let min_depth = compute_min_depth_to_leaf_filtered(graph, taxonomy);
    let mut diags = Vec::new();
    for (name, tax) in taxonomy {
        let is_dishonest = matches!(
            tax,
            FunctionTaxonomy::Dishonest(_) | FunctionTaxonomy::DeclaredDishonest
        );
        if !is_dishonest {
            continue;
        }
        let file = file_of
            .get(name)
            .cloned()
            .unwrap_or_else(|| Path::new("unknown").to_path_buf());
        let depth = min_depth.get(name).copied().unwrap_or(0);
        let is_entry = is_allowed_by_entry(&file, &ctx.entry_allowlist);
        if is_entry {
            if depth == 0 && ctx.honesty_depth > 0 {
                continue;
            }
            if ctx.honesty_depth > 0 && depth <= ctx.honesty_depth {
                continue;
            }
        } else if depth <= ctx.honesty_depth {
            continue;
        }
        let reason = match tax {
            FunctionTaxonomy::Dishonest(r) => r.to_string(),
            FunctionTaxonomy::DeclaredDishonest => "declared dishonest".to_string(),
            _ => String::new(),
        };
        diags.push(Diagnostic {
            file,
            line: 1,
            col: 0,
            code: "TAX001".to_string(),
            message: format!(
                "`{name}` is {reason} at depth {depth} > allowed {} — move dishonesty to leaf/entry (honesty_depth={})",
                ctx.honesty_depth, ctx.honesty_depth
            ),
        });
    }
    diags
}

#[allow(dead_code)]
fn compute_min_depth_to_leaf(graph: &CallGraph) -> HashMap<String, usize> {
    compute_min_depth_to_leaf_filtered(graph, &HashMap::new())
}

fn compute_min_depth_to_leaf_filtered(
    graph: &CallGraph,
    taxonomy: &HashMap<String, FunctionTaxonomy>,
) -> HashMap<String, usize> {
    let nodes = graph.all_nodes();
    let is_dishonest = |n: &String| {
        taxonomy.is_empty()
            || matches!(
                taxonomy.get(n),
                Some(FunctionTaxonomy::Dishonest(_)) | Some(FunctionTaxonomy::DeclaredDishonest)
            )
    };
    let has_dishonest_callee =
        |n: &String| graph.callees(n).iter().any(|c| is_dishonest(&c.clone()));
    let mut depth: HashMap<String, usize> = HashMap::new();
    let mut queue = VecDeque::new();
    for n in &nodes {
        if graph.callees(n).is_empty() || !has_dishonest_callee(n) {
            depth.insert(n.clone(), 0);
            queue.push_back(n.clone());
        }
    }
    while let Some(node) = queue.pop_front() {
        let Some(d) = depth.get(&node).copied() else {
            continue;
        };
        for caller in graph.callers(&node) {
            if !is_dishonest(caller) && !taxonomy.is_empty() {
                continue;
            }
            let Some(next) = d.checked_add(1) else {
                continue;
            };
            let entry = depth.entry(caller.clone()).or_insert(usize::MAX);
            if next < *entry {
                *entry = next;
                queue.push_back(caller.clone());
            }
        }
    }
    for n in nodes {
        depth.entry(n).or_insert(0);
    }
    depth
}

fn is_allowed_by_entry(file: &Path, allowlist: &[String]) -> bool {
    let s = file.to_string_lossy();
    for pat in allowlist {
        if let Some(prefix) = pat.strip_suffix("/**") {
            if s.starts_with(prefix) {
                return true;
            }
        } else if pat.contains('*') {
            if s.contains(pat.replace("**", "").replace('*', "").as_str()) {
                return true;
            }
            if glob_match(&s, pat) {
                return true;
            }
        } else if s == pat.as_str() || s.ends_with(pat.as_str()) {
            return true;
        }
    }
    false
}

fn glob_match(path: &str, pat: &str) -> bool {
    if pat == "examples/**" {
        return path.starts_with("examples/") || path.contains("/examples/");
    }
    if pat == "src/main.rs" {
        return path.ends_with("main.rs");
    }
    if pat == "src/testing/**" {
        return path.starts_with("testing/") || path.contains("/testing/");
    }
    false
}

pub fn taxonomy_from_reason(reason: DishonestReason) -> FunctionTaxonomy {
    FunctionTaxonomy::Dishonest(reason)
}

#[cfg(test)]
mod tests {
    use super::check_depth;
    use super::compute_min_depth_to_leaf;
    use crate::call_graph::CallGraph;
    use crate::taxonomy::DishonestReason;
    use crate::taxonomy::FunctionTaxonomy;
    use crate::taxonomy_ctx::TaxonomyCtx;
    use std::collections::HashMap;
    use std::path::PathBuf;

    #[test]
    fn test_usage_depth_zero_leaves_only() {
        let mut g = CallGraph::default();
        g.add_edge("main".to_string(), "render_frame".to_string());
        g.add_edge("render_frame".to_string(), "draw".to_string());
        let d = compute_min_depth_to_leaf(&g);
        assert_eq!(d.get("draw").copied().unwrap_or(99), 0);
        assert_eq!(d.get("render_frame").copied().unwrap_or(99), 1);
        assert_eq!(d.get("main").copied().unwrap_or(99), 2);
    }

    #[test]
    fn test_usage_allow_depth_one() {
        let mut g = CallGraph::default();
        g.add_edge("main".to_string(), "render_frame".to_string());
        g.add_edge("render_frame".to_string(), "draw".to_string());
        let mut tax = HashMap::new();
        tax.insert(
            "draw".to_string(),
            FunctionTaxonomy::Dishonest(DishonestReason::HiddenWrite("screen".to_string())),
        );
        tax.insert(
            "render_frame".to_string(),
            FunctionTaxonomy::Dishonest(DishonestReason::CallsDishonest("draw".to_string())),
        );
        let mut file_of = HashMap::new();
        file_of.insert("draw".to_string(), PathBuf::from("src/draw.rs"));
        file_of.insert("render_frame".to_string(), PathBuf::from("src/render.rs"));
        let ctx = TaxonomyCtx {
            honesty_depth: 1,
            entry_allowlist: vec![],
            ..Default::default()
        };
        let diags = check_depth(&g, &tax, &file_of, &ctx);
        assert!(
            diags.is_empty(),
            "depth 1 should allow render_frame at depth 1"
        );

        let ctx0 = TaxonomyCtx {
            honesty_depth: 0,
            entry_allowlist: vec![],
            ..Default::default()
        };
        let diags0 = check_depth(&g, &tax, &file_of, &ctx0);
        assert_eq!(diags0.len(), 1);
        assert!(diags0[0].message.contains("render_frame"));
    }

    #[test]
    fn test_usage_entry_allowlist() {
        let g = CallGraph::default();
        let mut tax = HashMap::new();
        tax.insert(
            "main_dishonest".to_string(),
            FunctionTaxonomy::DeclaredDishonest,
        );
        let mut file_of = HashMap::new();
        file_of.insert("main_dishonest".to_string(), PathBuf::from("src/main.rs"));
        let ctx = TaxonomyCtx::default();
        let diags = check_depth(&g, &tax, &file_of, &ctx);
        assert!(diags.is_empty());
    }
}
