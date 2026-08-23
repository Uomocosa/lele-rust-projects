/// Gracefully shuts the node down (close admission -> drain in-flight client ops -> send
/// `NodeEvent::Disconnect`), then waits for the node task to finish on its own. Unlike an
/// `abort()`, this lets open WebSocket client connections wind down normally instead of
/// being cut mid-response (which freenet logs as "node shut down while handling responses").
/// The on-disk store (e.g. the redb file lock) is released once the task has fully finished,
/// so a subsequent start can reuse the same directory.
pub async fn shutdown(mut this: crate::structs::test_node::TestNode) {
    this.shutdown_handle.shutdown().await;
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
