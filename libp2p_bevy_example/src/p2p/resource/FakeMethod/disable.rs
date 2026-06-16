use crate::p2p::resource::Fake;

pub fn disable(_fake: Fake) -> Fake {
    Fake { enabled: false }
}

#[cfg(test)]
mod tests {
    use crate::p2p::resource::Fake;

    #[test]
    fn test_usage() {
        let fake = Fake::new().disable();
        assert!(!fake.enabled);
    }
}
