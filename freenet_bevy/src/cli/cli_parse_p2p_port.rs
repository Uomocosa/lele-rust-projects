pub fn parse_p2p_port() -> u16 {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--p2p-port"
            && let Some(val) = args.next()
            && let Ok(port) = val.parse::<u16>()
        {
            return port;
        }
    }
    let socket =
        std::net::UdpSocket::bind((std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST), 0))
            .expect("failed to probe an available UDP port");
    socket
        .local_addr()
        .expect("failed to read assigned port")
        .port()
}

#[cfg(test)]
mod tests {
    use super::parse_p2p_port;

    #[test]
    fn test_usage() {
        let p = parse_p2p_port();
        assert!(p > 0);
    }
}
