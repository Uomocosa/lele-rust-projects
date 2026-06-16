use crate::p2p::resource::Fake;

pub fn new() -> Fake {
    Fake { enabled: true }
}

#[cfg(test)]
mod tests {
    use crate::p2p::resource::Fake;

    #[test]
    fn test_usage() {
        let fake = Fake::new();
        assert!(fake.enabled);
    }
}
