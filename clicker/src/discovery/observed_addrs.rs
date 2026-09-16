#[must_use]
pub fn observed_addrs(seen: Vec<String>, listen: &[String]) -> Vec<String> {
    let ports = listen_ports(listen);
    seen.into_iter()
        .filter(|addr| !addr.contains("0.0.0.0"))
        .filter(|addr| port_of(addr).is_some_and(|port| ports.contains(&port)))
        .collect()
}

// needed helper: collects listen ports so ephemeral outbound ports never match
fn listen_ports(listen: &[String]) -> Vec<String> {
    listen.iter().filter_map(|addr| port_of(addr)).collect()
}

// needed helper: extracts the tcp/udp port token from a multiaddr string
fn port_of(addr: &str) -> Option<String> {
    let mut parts = addr.split('/');
    while let Some(part) = parts.next() {
        if (part == "tcp" || part == "udp")
            && let Some(port) = parts.next()
        {
            return Some(port.to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::observed_addrs;

    #[test]
    fn test_usage() {
        let listen = vec![
            "/ip4/127.0.0.1/tcp/42515".to_string(),
            "/ip4/127.0.0.1/udp/38180/quic-v1".to_string(),
        ];
        let seen = vec!["/ip4/127.0.0.1/tcp/59700".to_string()];
        assert_eq!(observed_addrs(seen, &listen).len(), 0);
    }

    #[test]
    fn observed_matching_port_kept() {
        let listen = vec!["/ip4/127.0.0.1/tcp/42515".to_string()];
        let seen = vec!["/ip4/100.113.107.37/tcp/42515".to_string()];
        assert_eq!(observed_addrs(seen, &listen).len(), 1);
    }
}
