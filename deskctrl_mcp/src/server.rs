use std::sync::{Arc, atomic::AtomicU32};

use serde::Deserialize;

use rmcp::{
    ErrorData, ServerHandler,
    handler::server::wrapper::Parameters,
    model::{CallToolResult, ServerInfo},
    tool, tool_handler, tool_router,
};

use crate::{
    ClickParams, ProcessMap, ReadOutputParams, RecordVideoParams, Recording, ScreenshotParams,
    SendKeysParams, SendToTelegramParams, SpawnParams, WaitForOutputParams, WriteStdinParams,
    server_click_window, server_kill_process, server_method, server_send_keys,
    server_spawn_process, server_write_stdin,
};

#[derive(Clone)]
pub struct Server {
    pub processes: ProcessMap,
    pub next_id: Arc<AtomicU32>,
    pub artifacts_dir: Option<String>,
    pub bot_token: Option<String>,
    pub chat_id: Option<String>,
    pub recording: Arc<tokio::sync::Mutex<Option<Recording>>>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct PidParam {
    /// Managed process ID returned by spawn_process
    pub pid: u32,
    /// Send a step-by-step message to Telegram for this action. Default true; set false for
    /// routine kills you don't want in the feed.
    #[serde(default = "default_true")]
    pub send_to_telegram: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ActivateVpnParams {
    /// Specific server ID e.g. "IT#23" – optional, fastest if empty
    #[serde(default)]
    pub server: Option<String>,
    /// Country code or name e.g. "US" or "United States"
    #[serde(default)]
    pub country: Option<String>,
    /// City name e.g. "New York"
    #[serde(default)]
    pub city: Option<String>,
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
    pub fn with_artifacts_dir(artifacts_dir: Option<String>) -> Self { server_method::new(artifacts_dir) }

    #[tool(description = "Capture a screenshot. With no arguments, captures the whole screen; pass window_id (from list_windows), pid, or title to capture only that window. Returns a PNG image plus a text summary. Optional caption is sent to Telegram with the photo when send_to_telegram is true.")]
    async fn screenshot(&self, Parameters(params): Parameters<ScreenshotParams>) -> Result<CallToolResult, ErrorData> {
        let send_to_telegram = params.send_to_telegram;
        let result = server_method::screenshot(params, self.artifacts_dir.as_deref(), self.bot_token.as_deref(), self.chat_id.as_deref(), send_to_telegram).await.map_err(ErrorData::from)?;
        Ok(result)
    }

    #[tool(description = "Spawn a subprocess. Returns a numeric process ID you can pass to read_output, write_stdin, and kill_process.")]
    async fn spawn_process(&self, Parameters(params): Parameters<SpawnParams>) -> Result<CallToolResult, ErrorData> { server_spawn_process::spawn_process(self, params).await.map_err(ErrorData::from) }

    #[tool(description = "Drain buffered stdout/stderr collected since the last call. Waits up to timeout_ms for new output.")]
    async fn read_output(&self, Parameters(params): Parameters<ReadOutputParams>) -> Result<CallToolResult, ErrorData> { server_method::read_output(&self.processes, params).await.map_err(ErrorData::from) }

    #[tool(description = "Send text to a running process's stdin. A newline is appended automatically.")]
    async fn write_stdin(&self, Parameters(params): Parameters<WriteStdinParams>) -> Result<CallToolResult, ErrorData> { server_write_stdin::write_stdin(self, params).await.map_err(ErrorData::from) }

    #[tool(description = "Terminate a managed process and remove it from the list.")]
    async fn kill_process(&self, Parameters(params): Parameters<PidParam>) -> Result<CallToolResult, ErrorData> { server_kill_process::kill_process(self, params.pid, params.send_to_telegram).await.map_err(ErrorData::from) }

    #[tool(description = "List all managed processes with their IDs, command strings, and alive status.")]
    async fn list_processes(&self) -> Result<CallToolResult, ErrorData> { server_method::list_processes(&self.processes).await.map_err(ErrorData::from) }

    #[tool(description = "Block until a spawned process prints a line containing the given substring, or the timeout expires. Scans all output since spawn, including lines already returned by read_output. Timeout is capped at 120s — call again to keep waiting.")]
    async fn wait_for_output(&self, Parameters(params): Parameters<WaitForOutputParams>) -> Result<CallToolResult, ErrorData> { server_method::wait_for_output(&self.processes, params).await.map_err(ErrorData::from) }

    #[tool(description = "List the open desktop windows with their window IDs, owning PIDs, geometry, and titles. Pass a window_id to screenshot to capture just that window.")]
    async fn list_windows(&self) -> Result<CallToolResult, ErrorData> { server_method::list_windows().await.map_err(ErrorData::from) }

    #[tool(description = "Click inside a window at coordinates relative to its top-left corner (same coordinates as a screenshot of that window). Raises the window first, so it steals focus. Screenshot the window afterwards to confirm the click landed. Optional note is sent to Telegram when send_to_telegram is true.")]
    async fn click_window(&self, Parameters(params): Parameters<ClickParams>) -> Result<CallToolResult, ErrorData> { server_click_window::click_window(self, params).await.map_err(ErrorData::from) }

    #[tool(description = "Send a deliberate, bounded sequence of keyboard inputs into a window via XTEST. Takes window_id plus inputs, a non-empty ordered list of {type:tap|hold|chord|delay|text}. tap presses+releases one key; hold keeps a key down for duration_ms (use it for a run of repeated characters instead of many taps); chord presses several keys together (e.g. keys [\"ctrl\",\"shift\",\"esc\"]) and releases them all at once; delay waits duration_ms; text types literal printable ASCII (\\n is Enter, \\t is Tab). Raises the window first, so it steals focus. Screenshot the window afterwards to confirm the keys landed. Optional note is sent to Telegram when send_to_telegram is true.")]
    async fn send_keys(&self, Parameters(params): Parameters<SendKeysParams>) -> Result<CallToolResult, ErrorData> { server_send_keys::send_keys(self, params).await.map_err(ErrorData::from) }

    #[tool(description = "Send a text message and/or a base64-encoded PNG photo to a pre-configured Telegram chat.")]
    async fn send_to_telegram(&self, Parameters(params): Parameters<SendToTelegramParams>) -> Result<CallToolResult, ErrorData> {
        server_method::send_to_telegram(self.bot_token.as_deref(), self.chat_id.as_deref(), params).await.map_err(ErrorData::from)
    }

    #[tool(description = "Start or stop a screen recording and send it to Telegram. No args starts recording (whole screen, or the given window via window_id/pid/title); stop:true stops it, sends the MP4 to Telegram (caption = summary or an auto summary) and clears the recording.")]
    async fn record_video(&self, Parameters(params): Parameters<RecordVideoParams>) -> Result<CallToolResult, ErrorData> {
        server_method::record_video(&self.recording, self.bot_token.as_deref(), self.chat_id.as_deref(), self.artifacts_dir.as_deref(), params).await.map_err(ErrorData::from)
    }

    #[tool(description = "Check Proton VPN status: connection state, server, public IP, and default route. No sudo required; uses `protonvpn status` + curl + ip route.")]
    async fn check_vpn_status(&self) -> Result<CallToolResult, ErrorData> { server_method::vpn_status().await.map_err(ErrorData::from) }

    #[tool(description = "Activate Proton VPN (`protonvpn connect`). No sudo required. Prefer `protonvpn` binary; falls back to `protonvpn-cli` if needed. Optional server/country/city – all empty = fastest server. Already logged in; fails with auth error if session expired.")]
    async fn activate_vpn(&self, Parameters(params): Parameters<ActivateVpnParams>) -> Result<CallToolResult, ErrorData> { server_method::vpn_connect(params.server, params.country, params.city).await.map_err(ErrorData::from) }

    #[tool(description = "Deactivate Proton VPN (`protonvpn disconnect`). No sudo required.")]
    async fn deactivate_vpn(&self) -> Result<CallToolResult, ErrorData> { server_method::vpn_disconnect().await.map_err(ErrorData::from) }
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
        assert!(server.bot_token.is_none());
        assert!(server.chat_id.is_none());
        assert!(server.recording.blocking_lock().is_none());
    }
}
