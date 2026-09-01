use std::path::Path;

use super::start::start_node_at;

/// Starts an embedded network node against a caller-owned directory instead of a fresh
/// tempdir, so a later `start_at` call against the same directory can pick up whatever
/// the node previously persisted there (used to test restart persistence).
pub async fn start_at(
    dir: &Path,
) -> Result<crate::structs::test_node::TestNode, Box<dyn std::error::Error>> {
    let (port, task) = start_node_at(dir).await?;

    Ok(crate::structs::test_node::TestNode {
        _tmp: None,
        port,
        _task: task,
    })
}

#[cfg(test)]
mod tests {
    use crate::structs::test_node::TestNode;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_usage() {
        let dir = tempfile::tempdir().unwrap();
        let node = TestNode::start_at(dir.path()).await.unwrap();
        assert!(node.port() > 0);
    }
}
