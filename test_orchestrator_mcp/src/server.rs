use rmcp::{
    ErrorData, ServerHandler,
    handler::server::wrapper::Parameters,
    model::{CallToolResult, ServerInfo},
    tool, tool_handler, tool_router,
};

use crate::{
    DownloadArtifactsParams, GameStatusParams, LaunchGameParams, NextTagParams, ProbeNetworkParams,
    RunPipelineParams, RunStatusParams, StopGameParams, TriggerTagCiParams, server_method,
};

#[derive(Clone)]
pub struct Server {
    pub gh_repo: String,
    pub gh_token: Option<String>,
    pub game_exe: Option<String>,
}

impl Default for Server {
    fn default() -> Self {
        Self::new()
    }
}

#[tool_router]
#[rustfmt::skip]
impl Server {
    pub fn new() -> Self { server_method::new() }

    #[tool(description = "List the GitHub self-hosted runners registered for the repo with their status and labels. Shows both machines (Linux + Windows) from either PC.")]
    async fn list_runners(&self) -> Result<CallToolResult, ErrorData> { server_method::list_runners(&self.gh_repo, self.gh_token.as_deref()).await.map_err(ErrorData::from) }

    #[tool(description = "Trigger the self-hosted CI workflow (manual dispatch) on your own machines. Starts the Linux test gate and the Linux + Windows release builds, producing downloadable binaries with a single shared contract WASM. Set jobs=all (default, full pipeline), test, build, or cross-os to run only that stage.")]
    async fn run_pipeline(&self, Parameters(params): Parameters<RunPipelineParams>) -> Result<CallToolResult, ErrorData> { server_method::run_pipeline(&self.gh_repo, self.gh_token.as_deref(), params).await.map_err(ErrorData::from) }

    #[tool(description = "List the most recent GitHub Actions runs (both the self-hosted pipeline and the tag-triggered crate-tag CI) with their status.")]
    async fn list_runs(&self) -> Result<CallToolResult, ErrorData> { server_method::list_runs(&self.gh_repo, self.gh_token.as_deref()).await.map_err(ErrorData::from) }

    #[tool(description = "Show the status of a workflow run (defaults to the latest run) including per-job status and the run URL.")]
    async fn run_status(&self, Parameters(params): Parameters<RunStatusParams>) -> Result<CallToolResult, ErrorData> { server_method::run_status(&self.gh_repo, self.gh_token.as_deref(), params).await.map_err(ErrorData::from) }

    #[tool(description = "Download the artifacts (e.g. the Windows .exe or Linux binary) of a workflow run — defaults to the latest run — into a local directory.")]
    async fn download_artifacts(&self, Parameters(params): Parameters<DownloadArtifactsParams>) -> Result<CallToolResult, ErrorData> { server_method::download_artifacts(&self.gh_repo, self.gh_token.as_deref(), params).await.map_err(ErrorData::from) }

    #[tool(description = "Compute the next crate-tag CI tag (<crate>-<mode>-YYYY-MM-DD#N) without pushing anything. Modes: test, build, release, release-notests.")]
    async fn next_tag(&self, Parameters(params): Parameters<NextTagParams>) -> Result<CallToolResult, ErrorData> { server_method::next_tag(&self.gh_repo, self.gh_token.as_deref(), params).await.map(|tag| rmcp::model::CallToolResult::success(vec![rmcp::model::ContentBlock::text(tag)])).map_err(ErrorData::from) }

    #[tool(description = "Create and push a crate-tag CI tag (<crate>-<mode>-YYYY-MM-DD#N) via the GitHub API — this starts the tag-triggered GitHub-hosted workflow (test gate, build check, or full release). Use dry_run to preview the tag first.")]
    async fn trigger_tag_ci(&self, Parameters(params): Parameters<TriggerTagCiParams>) -> Result<CallToolResult, ErrorData> { server_method::trigger_tag_ci(&self.gh_repo, self.gh_token.as_deref(), params).await.map_err(ErrorData::from) }

    #[tool(description = "Launch the game on THIS machine with RUST_LOG=warn,roster=info,p2p=info, detached, logging to a file. Returns the pid. Uses the default dev/release binary unless exe is given.")]
    async fn launch_game(&self, Parameters(params): Parameters<LaunchGameParams>) -> Result<CallToolResult, ErrorData> { let exe = params.exe.as_deref().or(self.game_exe.as_deref()).map(str::to_owned); server_method::launch_game(exe.as_deref(), params).await.map_err(ErrorData::from) }

    #[tool(description = "Report the game's progress from its log file: ring connections, roster entries, libp2p connections, and any retry/error signals. Pass pid to also report whether the process is alive.")]
    async fn game_status(&self, Parameters(params): Parameters<GameStatusParams>) -> Result<CallToolResult, ErrorData> { server_method::game_status(params).await.map_err(ErrorData::from) }

    #[tool(description = "Terminate a running game process by pid.")]
    async fn stop_game(&self, Parameters(params): Parameters<StopGameParams>) -> Result<CallToolResult, ErrorData> { server_method::stop_game(params).await.map_err(ErrorData::from) }

    #[tool(description = "Trigger the network-probe workflow on both self-hosted runners, wait for it, and return each machine's public IP + LAN IP as JSON. Used to detect whether the two machines are on the same LAN.")]
    async fn probe_network(&self, Parameters(_params): Parameters<ProbeNetworkParams>) -> Result<CallToolResult, ErrorData> { server_method::probe_network(&self.gh_repo, self.gh_token.as_deref()).await.map_err(ErrorData::from) }
}

#[tool_handler]
#[rustfmt::skip]
impl ServerHandler for Server {
    fn get_info(&self) -> ServerInfo { server_method::get_info() }
}

#[cfg(test)]
mod tests {
    use rmcp::ServerHandler;

    use crate::Server;

    #[test]
    fn test_usage() {
        let server = Server::new();
        let info = server.get_info();
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
