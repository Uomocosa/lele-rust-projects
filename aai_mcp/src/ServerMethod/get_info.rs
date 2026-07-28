use rmcp::model::{Implementation, ServerCapabilities, ServerInfo};

pub fn get_info() -> ServerInfo {
    ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
        .with_server_info(Implementation::new("aai-mcp", env!("CARGO_PKG_VERSION")))
        .with_instructions(
            "AAI tools: screenshot (capture screen), spawn_process, read_output, \
             write_stdin, kill_process, list_processes.",
        )
}

#[cfg(test)]
mod tests {
    use super::get_info;

    #[test]
    fn test_usage() {
        let info = get_info();
        assert!(info.capabilities.tools.is_some());
        assert!(info.instructions.is_some_and(|i| i.contains("screenshot")));
        assert_eq!(info.server_info.name, "aai-mcp");
    }
}
