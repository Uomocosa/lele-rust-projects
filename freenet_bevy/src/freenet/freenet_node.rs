#[derive(Default)]
pub enum FreenetNode {
    #[default]
    Local,
    Remote {
        host: String,
        port: u16,
    },
}

#[cfg(test)]
mod tests {
    use super::FreenetNode;

    #[test]
    fn test_usage() {
        let node = FreenetNode::default();
        match node {
            FreenetNode::Local => {}
            FreenetNode::Remote { host, port } => {
                let _ = host;
                let _ = port;
            }
        }
    }
}
