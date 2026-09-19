use super::nat::Nat;

#[must_use]
pub fn dialable(nat: Nat, own_ip: &str, target_ip: &str) -> bool {
    match nat {
        Nat::Open => true,
        Nat::NoHairpin { .. } => own_ip != target_ip,
    }
}

#[cfg(test)]
mod tests {
    use super::Nat;
    use super::dialable;

    #[test]
    fn test_usage() {
        assert!(dialable(Nat::Open, "a", "a"));
        assert!(dialable(Nat::NoHairpin { public_ip: "x" }, "a", "b"));
        assert!(!dialable(Nat::NoHairpin { public_ip: "x" }, "a", "a"));
    }
}
