use crate::net_id;
#[must_use]
pub fn dialable(addrs: Vec<net_id::PeerAddr>) -> Vec<net_id::PeerAddr> {
    addrs
        .into_iter()
        .filter(|addr| !addr.contains("0.0.0.0"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::dialable;
    use crate::net_id;

    #[test]
    fn test_usage() {
        assert_eq!(
            dialable(vec![net_id::PeerAddr("/ip4/0.0.0.0/tcp/9000".to_string())]),
            Vec::<net_id::PeerAddr>::new()
        );
        assert_eq!(
            dialable(vec![net_id::PeerAddr("/ip4/1.2.3.4/tcp/9000".to_string())]).len(),
            1
        );
    }
}
