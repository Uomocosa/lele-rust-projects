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

#[cfg(test)]
mod tests {
    use super::DishonestReason;

    #[test]
    fn test_usage() {
        let r = DishonestReason::HiddenRead("clock".to_string());
        assert_eq!(r.to_string(), "hidden read: clock");
        assert_eq!(
            DishonestReason::CallsDishonest("draw".to_string()).to_string(),
            "calls dishonest: draw"
        );
    }
}
