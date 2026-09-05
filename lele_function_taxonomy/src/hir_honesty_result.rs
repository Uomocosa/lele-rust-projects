use crate::function_taxonomy::FunctionTaxonomy;

pub struct HirHonestyResult {
    pub taxonomy: FunctionTaxonomy,
    pub callees: Vec<String>,
}

#[cfg(feature = "rustc-private")]
pub mod rustc_impl {
    use std::collections::HashMap;
    use std::path::PathBuf;

    use crate::call_graph::CallGraph;
    use crate::function_taxonomy::FunctionTaxonomy;
    use crate::taxonomy_ctx::TaxonomyCtx;

    pub fn walk_crate(
        _tcx: (),
        _ctx: &TaxonomyCtx,
        _graph: &mut CallGraph,
        _taxonomy: &mut HashMap<String, FunctionTaxonomy>,
        _file_of: &mut HashMap<String, PathBuf>,
    ) {
        tracing::warn!("rustc-private walk_crate stub — wire TyCtxt walk for nightly 1.100 (api drift, keep stub compiling)");
    }
}

#[cfg(test)]
mod tests {
    use super::HirHonestyResult;
    use crate::function_taxonomy::FunctionTaxonomy;

    #[test]
    fn test_usage() {
        let r = HirHonestyResult {
            taxonomy: FunctionTaxonomy::Honest,
            callees: vec!["draw".to_string()],
        };
        assert_eq!(r.taxonomy, FunctionTaxonomy::Honest);
        assert_eq!(r.callees, vec!["draw".to_string()]);
    }
}
