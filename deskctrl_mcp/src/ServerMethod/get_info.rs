use rmcp::model::{Implementation, ServerCapabilities, ServerInfo};

pub fn get_info() -> ServerInfo {
    ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
        .with_server_info(Implementation::new(
            "deskctrl-mcp",
            env!("CARGO_PKG_VERSION"),
        ))
        .with_instructions(
            "deskctrl: desktop and process control. Windows: list_windows (open windows \
             with their IDs), screenshot (whole screen, or one window via \
             window_id/pid/title), click_window (click at window-relative coordinates). \
             Processes: spawn_process (returns an os_pid that matches list_windows, so a \
             GUI app you spawned can be screenshotted and clicked), read_output, \
             wait_for_output (block until a line appears), write_stdin, kill_process, \
             list_processes (managed subprocesses only). Also send_to_telegram, and \
             send_action_summary (sends a formatted Telegram message listing the clicks, \
             spawned processes, stdin writes, and kills done this session — call it when \
             wrapping up a task).",
        )
}

#[cfg(test)]
mod tests {
    use super::get_info;

    #[test]
    fn test_usage() {
        let info = get_info();
        assert!(info.capabilities.tools.is_some());
        // Every tool must be named here; this is the description clients surface.
        let instructions = info.instructions.unwrap_or_default();
        for tool in [
            "list_windows",
            "screenshot",
            "click_window",
            "spawn_process",
            "read_output",
            "wait_for_output",
            "write_stdin",
            "kill_process",
            "list_processes",
            "send_to_telegram",
            "send_action_summary",
        ] {
            assert!(instructions.contains(tool), "instructions omit {tool}");
        }
        assert_eq!(info.server_info.name, "deskctrl-mcp");
    }
}
