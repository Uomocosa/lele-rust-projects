use crate::dishonest_reason::DishonestReason;
use crate::function_taxonomy::FunctionTaxonomy;

pub fn taxonomy_from_reason(reason: DishonestReason) -> FunctionTaxonomy {
    FunctionTaxonomy::Dishonest(reason)
}

#[cfg(test)]
mod tests {
    use super::taxonomy_from_reason;
    use crate::dishonest_reason::DishonestReason;
    use crate::function_taxonomy::FunctionTaxonomy;

    #[test]
    fn test_usage() {
        let reason = DishonestReason::HiddenRead("clock".to_string());
        assert_eq!(
            taxonomy_from_reason(reason.clone()),
            FunctionTaxonomy::Dishonest(reason)
        );
    }
}
