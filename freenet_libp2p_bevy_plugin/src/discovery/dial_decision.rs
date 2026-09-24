#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialDecision {
    Wait,
    Dial,
    ForceDial,
}

#[cfg(test)]
mod tests {
    use super::DialDecision;

    #[test]
    fn test_usage() {
        assert_ne!(DialDecision::Wait, DialDecision::Dial);
        assert_eq!(DialDecision::ForceDial, DialDecision::ForceDial);
    }
}
