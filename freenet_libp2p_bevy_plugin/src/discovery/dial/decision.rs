#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    Wait,
    Dial,
    ForceDial,
}

#[cfg(test)]
mod tests {
    use super::Decision;

    #[test]
    fn test_usage() {
        assert_ne!(Decision::Wait, Decision::Dial);
        assert_eq!(Decision::ForceDial, Decision::ForceDial);
    }
}
