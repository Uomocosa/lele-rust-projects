use crate::dishonest_reason::DishonestReason;
use crate::function_taxonomy::FunctionTaxonomy;
use crate::hir_honesty_result::HirHonestyResult;
use crate::taxonomy_ctx::TaxonomyCtx;

pub fn analyze_stub(
    name: &str,
    ctx: &TaxonomyCtx,
    has_hidden_read: bool,
    has_hidden_write: bool,
    is_const: bool,
    callees: Vec<String>,
    callee_taxonomies: &std::collections::HashMap<
        String,
        FunctionTaxonomy,
        std::collections::hash_map::RandomState,
    >,
) -> HirHonestyResult {
    if ctx.is_declared_honest(name) {
        return HirHonestyResult {
            taxonomy: FunctionTaxonomy::DeclaredHonest,
            callees,
        };
    }
    if ctx.is_declared_dishonest(name) {
        return HirHonestyResult {
            taxonomy: FunctionTaxonomy::DeclaredDishonest,
            callees,
        };
    }
    if is_const {
        return HirHonestyResult {
            taxonomy: FunctionTaxonomy::Pure,
            callees,
        };
    }
    for callee in &callees {
        if let Some(tax) = callee_taxonomies.get(callee) {
            if matches!(
                tax,
                FunctionTaxonomy::Dishonest(_) | FunctionTaxonomy::DeclaredDishonest
            ) {
                return HirHonestyResult {
                    taxonomy: FunctionTaxonomy::Dishonest(DishonestReason::CallsDishonest(
                        callee.clone(),
                    )),
                    callees,
                };
            }
        }
        if ctx.is_declared_dishonest(callee) {
            return HirHonestyResult {
                taxonomy: FunctionTaxonomy::Dishonest(DishonestReason::CallsDishonest(
                    callee.clone(),
                )),
                callees,
            };
        }
    }
    if has_hidden_read {
        return HirHonestyResult {
            taxonomy: FunctionTaxonomy::Dishonest(DishonestReason::HiddenRead(name.to_string())),
            callees,
        };
    }
    if has_hidden_write {
        return HirHonestyResult {
            taxonomy: FunctionTaxonomy::Dishonest(DishonestReason::HiddenWrite(name.to_string())),
            callees,
        };
    }
    HirHonestyResult {
        taxonomy: FunctionTaxonomy::Honest,
        callees,
    }
}

#[cfg(test)]
mod tests {
    use super::analyze_stub;
    use crate::dishonest_reason::DishonestReason;
    use crate::function_taxonomy::FunctionTaxonomy;
    use crate::taxonomy_ctx::TaxonomyCtx;
    use std::collections::HashMap;

    #[test]
    fn test_usage() {
        let ctx = TaxonomyCtx::default();
        let r = analyze_stub("clear", &ctx, false, false, false, vec![], &HashMap::new());
        assert_eq!(r.taxonomy, FunctionTaxonomy::Honest);
    }

    #[test]
    fn test_usage_pure_const() {
        let ctx = TaxonomyCtx::default();
        let r = analyze_stub("add", &ctx, false, false, true, vec![], &HashMap::new());
        assert_eq!(r.taxonomy, FunctionTaxonomy::Pure);
    }

    #[test]
    fn test_usage_dishonest_hidden_read() {
        let ctx = TaxonomyCtx::default();
        let r = analyze_stub(
            "get_time",
            &ctx,
            true,
            false,
            false,
            vec![],
            &HashMap::new(),
        );
        assert!(matches!(
            r.taxonomy,
            FunctionTaxonomy::Dishonest(DishonestReason::HiddenRead(_))
        ));
    }

    #[test]
    fn test_usage_declared_overrides() {
        let mut ctx = TaxonomyCtx::default();
        ctx.declared_dishonest.insert("my_time".to_string());
        let r = analyze_stub(
            "my_time",
            &ctx,
            false,
            false,
            false,
            vec![],
            &HashMap::new(),
        );
        assert_eq!(r.taxonomy, FunctionTaxonomy::DeclaredDishonest);
    }

    #[test]
    fn test_usage_infectious() {
        let ctx = TaxonomyCtx::default();
        let mut map = HashMap::new();
        map.insert(
            "draw".to_string(),
            FunctionTaxonomy::Dishonest(DishonestReason::HiddenWrite("screen".to_string())),
        );
        let r = analyze_stub(
            "render",
            &ctx,
            false,
            false,
            false,
            vec!["draw".to_string()],
            &map,
        );
        assert!(matches!(
            r.taxonomy,
            FunctionTaxonomy::Dishonest(DishonestReason::CallsDishonest(_))
        ));
    }
}
