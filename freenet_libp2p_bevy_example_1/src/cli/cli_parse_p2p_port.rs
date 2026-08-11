use std::io;
use std::net::{IpAddr, Ipv4Addr, UdpSocket};

pub fn parse_p2p_port() -> io::Result<u16> {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--p2p-port"
            && let Some(val) = args.next()
            && let Ok(port) = val.parse::<u16>()
        {
            return Ok(port);
        }
    }
    probe_free_udp_port()
}

fn probe_free_udp_port() -> io::Result<u16> {
    let socket = UdpSocket::bind((IpAddr::V4(Ipv4Addr::LOCALHOST), 0))?;
    Ok(socket.local_addr()?.port())
}

#[cfg(test)]
mod tests {
    use super::parse_p2p_port;

    #[test]
    fn test_usage() {
        assert!(parse_p2p_port().is_ok_and(|port| port > 0));
    }
}
