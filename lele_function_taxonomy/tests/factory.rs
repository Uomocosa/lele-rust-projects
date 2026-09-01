use std::collections::HashMap;
use std::path::Path;

use lele_function_taxonomy::hir_visitor::analyze_stub;
use lele_function_taxonomy::taxonomy::FunctionTaxonomy;
use lele_function_taxonomy::taxonomy_ctx::TaxonomyCtx;

fn is_const_fn(content: &str) -> bool {
    content.contains("const fn")
}

fn has_hidden_pattern(content: &str) -> bool {
    content.contains("SystemTime::now")
        || content.contains("Instant::now")
        || content.contains("thread_rng")
        || content.contains("getrandom")
        || content.contains("std::env::")
        || content.contains("std::fs::")
        || content.contains("static ")
        || content.contains("GLOBAL")
}

fn file_stem(path: &Path) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string()
}

#[test]
fn test_usage_factory_pure() {
    let ctx = TaxonomyCtx::default();
    let dir = Path::new("tests/pure_functions");
    for entry in walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(Result::ok)
    {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        let content = std::fs::read_to_string(path).unwrap();
        let stem = file_stem(path);
        let is_const = is_const_fn(&content);
        assert!(
            is_const,
            "pure file {} must contain `const fn` (pure = const only)",
            path.display()
        );
        let r = analyze_stub(&stem, &ctx, false, false, is_const, vec![], &HashMap::new());
        assert_eq!(
            r.taxonomy,
            FunctionTaxonomy::Pure,
            "pure file {} expected Pure, got {:?}",
            path.display(),
            r.taxonomy
        );
    }
}

#[test]
fn test_usage_factory_honest() {
    let ctx = TaxonomyCtx::default();
    let dir = Path::new("tests/honest_functions");
    for entry in walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(Result::ok)
    {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        let content = std::fs::read_to_string(path).unwrap();
        let stem = file_stem(path);
        let is_const = is_const_fn(&content);
        assert!(
            !is_const,
            "honest file {} must not be const fn (pure is const-only)",
            path.display()
        );
        let has_hidden = has_hidden_pattern(&content);
        assert!(
            !has_hidden,
            "honest file {} must not contain hidden patterns",
            path.display()
        );
        let r = analyze_stub(&stem, &ctx, false, false, is_const, vec![], &HashMap::new());
        assert_eq!(
            r.taxonomy,
            FunctionTaxonomy::Honest,
            "honest file {} expected Honest, got {:?}",
            path.display(),
            r.taxonomy
        );
    }
}

#[test]
fn test_usage_factory_dishonest() {
    let ctx = TaxonomyCtx::default();
    let dir = Path::new("tests/dishonest_functions");
    for entry in walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(Result::ok)
    {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        let content = std::fs::read_to_string(path).unwrap();
        let stem = file_stem(path);
        let has_hidden = has_hidden_pattern(&content);
        let callees = if content.contains("get_time_inner") {
            vec!["get_time_inner".to_string()]
        } else {
            vec![]
        };
        let mut callee_map = HashMap::new();
        if callees.contains(&"get_time_inner".to_string()) {
            callee_map.insert(
                "get_time_inner".to_string(),
                FunctionTaxonomy::Dishonest(
                    lele_function_taxonomy::taxonomy::DishonestReason::HiddenRead(
                        "time".to_string(),
                    ),
                ),
            );
        }
        let has_hidden_read = has_hidden || !callees.is_empty();
        let r = analyze_stub(
            &stem,
            &ctx,
            has_hidden_read,
            false,
            false,
            callees,
            &callee_map,
        );
        assert!(
            matches!(
                r.taxonomy,
                FunctionTaxonomy::Dishonest(_) | FunctionTaxonomy::DeclaredDishonest
            ),
            "dishonest file {} expected Dishonest, got {:?}",
            path.display(),
            r.taxonomy
        );
    }
}
