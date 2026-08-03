pub struct Score {
    pub value: u32,
}

impl Score {
    pub fn value(&self) -> u32 {   // VIOLATION: trivial getter
        self.value
    }
}

#[cfg(test)]
mod tests {
    use super::Score;

    #[test]
    fn test_usage() {
        let s = Score { value: 42 };
        assert_eq!(s.value(), 42);
    }
}
