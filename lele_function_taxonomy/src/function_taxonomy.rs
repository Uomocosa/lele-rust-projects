use crate::dishonest_reason::DishonestReason;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FunctionTaxonomy {
    Pure,
    Honest,
    Dishonest(DishonestReason),
    DeclaredHonest,
    DeclaredDishonest,
    Unknown { reason: String },
}

impl std::fmt::Display for FunctionTaxonomy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pure => write!(f, "Pure"),
            Self::Honest => write!(f, "Honest"),
            Self::Dishonest(r) => write!(f, "Dishonest({r})"),
            Self::DeclaredHonest => write!(f, "DeclaredHonest"),
            Self::DeclaredDishonest => write!(f, "DeclaredDishonest"),
            Self::Unknown { reason } => write!(f, "Unknown({reason})"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::FunctionTaxonomy;
    use crate::dishonest_reason::DishonestReason;

    #[test]
    fn test_usage() {
        assert_eq!(FunctionTaxonomy::Pure.to_string(), "Pure");
        assert_eq!(FunctionTaxonomy::Honest.to_string(), "Honest");
        let d = FunctionTaxonomy::Dishonest(DishonestReason::HiddenRead("clock".to_string()));
        assert_eq!(d.to_string(), "Dishonest(hidden read: clock)");
    }
}
