/// Aborts the node's task and waits for it to actually finish, so its on-disk store (e.g. the
/// redb file lock) is released before a subsequent start reuses the same directory.
pub async fn shutdown(mut this: crate::structs::test_node::TestNode) {
    this._task.abort();
    let _ = (&mut this._task).await;
}

#[cfg(test)]
mod tests {
    use crate::structs::test_node::TestNode;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_usage() {
        let node = TestNode::start_gateway(0).await.unwrap();
        node.shutdown().await;
    }
}
