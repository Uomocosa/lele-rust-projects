use freenet::config::{ConfigArgs, ConfigPathsArgs, NetworkArgs};

use freenet::local_node::OperationMode;

/// Build `ConfigArgs` for a test node.
#[must_use]
pub fn node_args(tmp: &tempfile::TempDir, network: NetworkArgs) -> ConfigArgs {
    ConfigArgs {
        mode: Some(OperationMode::Network),
        network_api: network,
        config_paths: ConfigPathsArgs {
            config_dir: Some(tmp.path().to_path_buf()),
            data_dir: Some(tmp.path().to_path_buf()),
            log_dir: Some(tmp.path().to_path_buf()),
        },
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::node_args;
    use freenet::config::NetworkArgs;

    #[test]
    fn test_usage() {
        let tmp = tempfile::tempdir().unwrap();
        let args = node_args(&tmp, NetworkArgs::default());
        assert!(args.config_paths.config_dir.is_some());
    }
}
