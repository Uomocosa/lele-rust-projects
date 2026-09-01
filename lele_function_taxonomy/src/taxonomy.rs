use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub enum DishonestReason {
    HiddenRead(String),
    HiddenWrite(String),
    Nondeterministic(String),
    CallsDishonest(String),
}

impl std::fmt::Display for DishonestReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HiddenRead(s) => write!(f, "hidden read: {s}"),
            Self::HiddenWrite(s) => write!(f, "hidden write: {s}"),
            Self::Nondeterministic(s) => write!(f, "nondeterministic: {s}"),
            Self::CallsDishonest(s) => write!(f, "calls dishonest: {s}"),
        }
    }
}

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

// Rust Pure = `const fn` only. All non-const fns are at most Honest.
// no test_usage necessary
