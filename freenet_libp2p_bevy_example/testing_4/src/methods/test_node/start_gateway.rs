use super::start_node_at::{free_udp_port, start_node_at};

pub async fn start_gateway(
    public_port: u16,
) -> Result<crate::structs::test_node::TestNode, Box<dyn std::error::Error>> {
    let tmp = tempfile::tempdir()?;
    let public_port = if public_port == 0 {
        free_udp_port()?
    } else {
        public_port
    };
    let (port, public_key_hex, task, shutdown_handle) =
        start_node_at(tmp.path(), true, public_port, None).await?;

    Ok(crate::structs::test_node::TestNode {
        _tmp: Some(tmp),
        port,
        public_port,
        public_key_hex,
        _task: task,
        shutdown_handle,
    })
}

#[cfg(test)]
mod tests {
    use crate::structs::test_node::TestNode;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_usage() {
        let node = TestNode::start_gateway(0).await.unwrap();
        assert!(node.port() > 0);
        assert_eq!(node.public_key_hex().len(), 64);
        node.shutdown().await;
    }
}
