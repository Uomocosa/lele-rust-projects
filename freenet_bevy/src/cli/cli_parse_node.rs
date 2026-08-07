use crate::freenet;

pub fn parse_node() -> freenet::FreenetNode {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--node" {
            match args.next().as_deref() {
                Some("local") => return freenet::FreenetNode::Local,
                Some("remote") => {
                    let host = std::env::var("FREENET_HOST").unwrap_or_else(|_| "127.0.0.1".into());
                    let port: u16 = std::env::var("FREENET_PORT")
                        .ok()
                        .and_then(|v| v.parse().ok())
                        .unwrap_or(7509);
                    return freenet::FreenetNode::Remote { host, port };
                }
                _ => {}
            }
        }
    }
    freenet::FreenetNode::default()
}

#[cfg(test)]
mod tests {
    use super::parse_node;

    #[test]
    fn test_usage() {
        let node = parse_node();
        match node {
            crate::freenet::FreenetNode::Local => {}
            crate::freenet::FreenetNode::Remote { host, port } => {
                let _ = host;
                let _ = port;
            }
        }
    }
}
