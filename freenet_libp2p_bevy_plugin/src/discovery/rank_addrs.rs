use crate::p2p;

#[must_use]
pub fn rank_addrs(addrs: &[String], mode: p2p::TransportMode) -> Vec<String> {
    let mut scored: Vec<(u8, u8, &String)> = Vec::new();
    for addr in addrs {
        if !transport_allowed(addr, mode) {
            continue;
        }
        scored.push((transport_rank(addr), net_rank(addr), addr));
    }
    scored.sort_by_key(|a| (a.0, a.1));
    scored
        .into_iter()
        .map(|(_, _, addr)| addr.clone())
        .collect()
}

// needed helper: drops addrs of transports disabled by the mode
fn transport_allowed(addr: &str, mode: p2p::TransportMode) -> bool {
    match mode {
        p2p::TransportMode::Tcp => !addr.contains("/udp/"),
        p2p::TransportMode::Quic => addr.contains("/udp/"),
        p2p::TransportMode::Both => true,
    }
}

// needed helper: prefers QUIC transports so real-world hole punching wins
fn transport_rank(addr: &str) -> u8 {
    if addr.contains("/udp/") {
        return 0;
    }
    1
}

// needed helper: ranks loopback above direct LAN above CGNAT relay-style paths
fn net_rank(addr: &str) -> u8 {
    if addr.contains("127.") || addr.contains("::1") {
        return 0;
    }
    if addr.contains("/ip4/100.") {
        return 2;
    }
    1
}

#[cfg(test)]
mod tests {
    use super::rank_addrs;
    use crate::p2p;

    #[test]
    fn test_usage() {
        let addrs = vec![
            "/ip4/192.168.1.9/tcp/4001".to_string(),
            "/ip4/192.168.1.9/udp/4001/quic-v1".to_string(),
            "/ip4/127.0.0.1/tcp/4001".to_string(),
        ];
        assert_eq!(
            rank_addrs(&addrs, p2p::TransportMode::Both),
            vec![
                "/ip4/192.168.1.9/udp/4001/quic-v1".to_string(),
                "/ip4/127.0.0.1/tcp/4001".to_string(),
                "/ip4/192.168.1.9/tcp/4001".to_string(),
            ]
        );
        assert_eq!(rank_addrs(&addrs, p2p::TransportMode::Tcp).len(), 2);
        assert_eq!(rank_addrs(&addrs, p2p::TransportMode::Quic).len(), 1);
    }
}
