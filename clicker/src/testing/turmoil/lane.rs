use super::lane_new;

pub struct Lane {
    pub name: &'static str,
    pub own: u64,
    pub clicks: u32,
    pub visible: &'static [&'static str],
}

#[rustfmt::skip]
impl Lane {
    #[must_use]
    pub const fn new(
        name: &'static str,
        own: u64,
        clicks: u32,
        visible: &'static [&'static str],
    ) -> Self { lane_new::new(name, own, clicks, visible) }
}

#[cfg(test)]
mod tests {
    use super::Lane;

    #[test]
    fn test_usage() {
        let lane = Lane::new("peer-1", 1, 1, &["peer-2"]);
        assert_eq!(lane.own, 1);
    }
}
