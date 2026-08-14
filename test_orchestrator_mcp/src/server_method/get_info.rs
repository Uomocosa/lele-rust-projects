use rmcp::model::{Implementation, ServerCapabilities, ServerInfo};

pub fn get_info() -> ServerInfo {
    ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
        .with_server_info(Implementation::new(
            "test-orchestrator-mcp",
            env!("CARGO_PKG_VERSION"),
        ))
        .with_instructions(
            "test-orchestrator-mcp: drive the cross-machine test pipeline for \
             freenet-libp2p-bevy-example-1 from either PC. CI tools (work on both \
             machines, via the gh CLI + GH_TOKEN from .env): list_runners (self-hosted \
             runner status), run_pipeline (trigger the self-hosted CI workflow on your \
             machines), list_runs, run_status, download_artifacts (grab the built \
             binaries), next_tag (preview the next crate-tag CI tag), trigger_tag_ci \
             (push a <crate>-<mode>-YYYY-MM-DD#N tag to start the GitHub-hosted test / \
             build / release workflow). Runtime tools (local to the machine the MCP runs \
             on): launch_game (start the game detached with RUST_LOG=warn,roster=info,\
             p2p=info, log to file), game_status (grep the log for ring connections, \
             roster entries, libp2p connections, errors), stop_game, probe_network \
             (trigger the network-probe workflow and return each machine's public + LAN \
             IP to detect a same-LAN run). Secrets: GH_TOKEN lives only in <crate>/.env, \
             never in code.",
        )
}

#[cfg(test)]
mod tests {
    use super::get_info;

    #[test]
    fn test_usage() {
        let info = get_info();
        assert!(info.capabilities.tools.is_some());
        let instructions = info.instructions.unwrap_or_default();
        for tool in [
            "list_runners",
            "run_pipeline",
            "list_runs",
            "run_status",
            "download_artifacts",
            "next_tag",
            "trigger_tag_ci",
            "launch_game",
            "game_status",
            "stop_game",
            "probe_network",
        ] {
            assert!(instructions.contains(tool), "instructions omit {tool}");
        }
        assert_eq!(info.server_info.name, "test-orchestrator-mcp");
    }
}
