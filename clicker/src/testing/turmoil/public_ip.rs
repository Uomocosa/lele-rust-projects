use super::nat::Nat;

#[must_use]
pub const fn public_ip(nat: Nat, name: &'static str) -> &'static str {
    match nat {
        Nat::Open => name,
        Nat::NoHairpin { public_ip } => public_ip,
    }
}

#[cfg(test)]
mod tests {
    use super::Nat;
    use super::public_ip;

    #[test]
    fn test_usage() {
        assert_eq!(public_ip(Nat::Open, "peer-1"), "peer-1");
        assert_eq!(
            public_ip(
                Nat::NoHairpin {
                    public_ip: "1.2.3.4"
                },
                "peer-1"
            ),
            "1.2.3.4"
        );
    }
}
