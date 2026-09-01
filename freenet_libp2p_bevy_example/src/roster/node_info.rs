pub struct NodeInfo {
    pub host: String,
    pub ws_port: u16,
    pub public_port: u16,
    pub public_key_hex: String,
    pub node_dir: tempfile::TempDir,
}
