use super::lane::Lane;

#[must_use]
pub const fn new(
    name: &'static str,
    own: u64,
    clicks: u32,
    visible: &'static [&'static str],
) -> Lane {
    Lane {
        name,
        own,
        clicks,
        visible,
    }
}

#[cfg(test)]
mod tests {
    use super::new;

    #[test]
    fn test_usage() {
        let lane = new("peer-2", 2, 5, &[]);
        assert_eq!(lane.name, "peer-2");
    }
}
