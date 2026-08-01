use std::sync::{Arc, atomic::AtomicU32};

use rmcp::{
    ErrorData, ServerHandler,
    handler::server::wrapper::Parameters,
    model::{CallToolResult, ServerInfo},
    tool, tool_handler, tool_router,
};

use crate::{
    PidParam, ProcessMap, ReadOutputParams, SendToTelegramParams, ServerMethod, SpawnParams,
    WriteStdinParams,
};

#[derive(Clone)]
pub struct Server {
    pub processes: ProcessMap,
    pub next_id: Arc<AtomicU32>,
    pub artifacts_dir: Option<String>,
    pub bot_token: Option<String>,
    pub chat_id: Option<String>,
}

impl Default for Server {
    fn default() -> Self {
        Self::new()
    }
}

#[tool_router]
#[rustfmt::skip]
impl Server {
    pub fn new() -> Self { Self::with_artifacts_dir(None) }

    #[rustfmt::skip]
    pub fn with_artifacts_dir(artifacts_dir: Option<String>) -> Self { ServerMethod::new(artifacts_dir) }

    #[tool(description = "Capture a screenshot of the primary monitor. Returns a PNG image plus a text summary.")]
    async fn screenshot(&self) -> Result<CallToolResult, ErrorData> { ServerMethod::screenshot(self.artifacts_dir.as_deref(), self.bot_token.as_deref(), self.chat_id.as_deref()).await.map_err(ErrorData::from) }

    #[tool(description = "Spawn a subprocess. Returns a numeric process ID you can pass to read_output, write_stdin, and kill_process.")]
    async fn spawn_process(&self, Parameters(params): Parameters<SpawnParams>) -> Result<CallToolResult, ErrorData> { ServerMethod::spawn_process(&self.processes, &self.next_id, params).await.map_err(ErrorData::from) }

    #[tool(description = "Drain buffered stdout/stderr collected since the last call. Waits up to timeout_ms for new output.")]
    async fn read_output(&self, Parameters(params): Parameters<ReadOutputParams>) -> Result<CallToolResult, ErrorData> { ServerMethod::read_output(&self.processes, params).await.map_err(ErrorData::from) }

    #[tool(description = "Send text to a running process's stdin. A newline is appended automatically.")]
    async fn write_stdin(&self, Parameters(params): Parameters<WriteStdinParams>) -> Result<CallToolResult, ErrorData> { ServerMethod::write_stdin(&self.processes, params).await.map_err(ErrorData::from) }

    #[tool(description = "Terminate a managed process and remove it from the list.")]
    async fn kill_process(&self, Parameters(params): Parameters<PidParam>) -> Result<CallToolResult, ErrorData> { ServerMethod::kill_process(&self.processes, params.pid).await.map_err(ErrorData::from) }

    #[tool(description = "List all managed processes with their IDs, command strings, and alive status.")]
    async fn list_processes(&self) -> Result<CallToolResult, ErrorData> { ServerMethod::list_processes(&self.processes).await.map_err(ErrorData::from) }

    #[tool(description = "Send a text message and/or a base64-encoded PNG photo to a pre-configured Telegram chat.")]
    async fn send_to_telegram(&self, Parameters(params): Parameters<SendToTelegramParams>) -> Result<CallToolResult, ErrorData> { ServerMethod::send_to_telegram(self.bot_token.as_deref(), self.chat_id.as_deref(), params).await.map_err(ErrorData::from) }
}

#[tool_handler]
#[rustfmt::skip]
impl ServerHandler for Server {
    fn get_info(&self) -> ServerInfo { ServerMethod::get_info() }
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
        assert!(server.bot_token.is_none());
        assert!(server.chat_id.is_none());
    }
}
