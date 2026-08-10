use super::start_node_at::{free_udp_port, start_node_at};

pub async fn start_peer(
    gateway_public_port: u16,
    gateway_public_key_hex: &str,
) -> Result<crate::structs::test_node::TestNode, Box<dyn std::error::Error>> {
    let tmp = tempfile::tempdir()?;
    let public_port = free_udp_port()?;
    let gateway = format!("127.0.0.1:{gateway_public_port},{gateway_public_key_hex}");
    let (port, public_key_hex, task) =
        start_node_at(tmp.path(), false, public_port, Some(gateway)).await?;

    Ok(crate::structs::test_node::TestNode {
        _tmp: Some(tmp),
        port,
        public_port,
        public_key_hex,
        _task: task,
    })
}
// no test_usage necessary — needs a live gateway, exercised by the two-node roster test
