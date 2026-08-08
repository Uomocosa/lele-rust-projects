use rmcp::model::{Implementation, ServerCapabilities, ServerInfo};

pub fn get_info() -> ServerInfo {
    ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
        .with_server_info(Implementation::new(
            "deskctrl-mcp",
            env!("CARGO_PKG_VERSION"),
        ))
        .with_instructions(
            "deskctrl: desktop and process control. list_windows (open windows with their \
             IDs), screenshot (whole screen, or one window via window_id/pid/title), \
             spawn_process, read_output, write_stdin, kill_process, list_processes \
             (managed subprocesses only), send_to_telegram.",
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
        assert_eq!(info.server_info.name, "deskctrl-mcp");
    }
}
