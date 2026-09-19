#[must_use]
pub fn own_of(name: &str) -> u64 {
    match name {
        "peer-1" => 1,
        "peer-2" => 2,
        _ => 3,
    }
}

#[cfg(test)]
mod tests {
    use super::own_of;

    #[test]
    fn test_usage() {
        assert_eq!(own_of("peer-1"), 1);
        assert_eq!(own_of("peer-3"), 3);
    }
}
