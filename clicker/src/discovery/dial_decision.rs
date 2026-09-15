#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialDecision {
    Dial,
    ForceDial,
    Wait,
}

#[cfg(test)]
mod tests {
    use super::DialDecision;

    #[test]
    fn test_usage() {
        assert_eq!(DialDecision::Dial, DialDecision::Dial);
        assert_ne!(DialDecision::Dial, DialDecision::Wait);
        assert_ne!(DialDecision::ForceDial, DialDecision::Wait);
    }
}
