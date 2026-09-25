use super::super::params::node_mode::NodeMode;
use super::bootstrap_node::bootstrap_node;

pub async fn start_node(
    mode: NodeMode,
) -> Result<(Option<tempfile::TempDir>, u16), super::super::error::Error> {
    match mode {
        NodeMode::Embedded => {
            let (guard, port) = bootstrap_node().await?;
            Ok((Some(guard), port))
        }
        NodeMode::External { ws_port } => Ok((None, ws_port)),
    }
}

// no test_usage necessary
