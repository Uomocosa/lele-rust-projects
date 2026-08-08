use std::sync::{Arc, Mutex, atomic::AtomicU32};

use rmcp::{
    ErrorData, ServerHandler,
    handler::server::wrapper::Parameters,
    model::{CallToolResult, ServerInfo},
    tool, tool_handler, tool_router,
};

use crate::{
    ClickParams, PidParam, ProcessMap, ReadOutputParams, ScreenshotParams, SendToTelegramParams,
    ServerMethod, SpawnParams, WaitForOutputParams, WriteStdinParams,
};

#[derive(Clone)]
pub struct Server {
    pub processes: ProcessMap,
    pub next_id: Arc<AtomicU32>,
    pub artifacts_dir: Option<String>,
    pub bot_token: Option<String>,
    pub chat_id: Option<String>,
    pub action_log: Arc<Mutex<Vec<String>>>,
    pub last_screenshot: Arc<Mutex<Option<Vec<u8>>>>,
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

    fn log_action(&self, entry: String) { self.action_log.lock().unwrap().push(entry); }

    #[tool(description = "Capture a screenshot. With no arguments, captures the whole screen; pass window_id (from list_windows), pid, or title to capture only that window. Returns a PNG image plus a text summary.")]
    async fn screenshot(&self, Parameters(params): Parameters<ScreenshotParams>) -> Result<CallToolResult, ErrorData> {
        let target = params.window_id.clone().or(params.title.clone()).or(params.pid.map(|p| p.to_string())).unwrap_or_else(|| "full screen".to_string());
        let result = ServerMethod::screenshot(params, self.artifacts_dir.as_deref(), self.bot_token.as_deref(), self.chat_id.as_deref(), Some(&self.last_screenshot)).await.map_err(ErrorData::from)?;
        self.log_action(format!("\u{1F4F8} <b>screenshot</b>: {}", html_escape(&target)));
        Ok(result)
    }

    #[tool(description = "Spawn a subprocess. Returns a numeric process ID you can pass to read_output, write_stdin, and kill_process.")]
    async fn spawn_process(&self, Parameters(params): Parameters<SpawnParams>) -> Result<CallToolResult, ErrorData> {
        let desc = format!("{} {}", params.cmd, params.args.join(" "));
        let result = ServerMethod::spawn_process(&self.processes, &self.next_id, params).await.map_err(ErrorData::from)?;
        self.log_action(format!("\u{1F680} <b>spawn_process</b>: \"{}\"", html_escape(desc.trim())));
        Ok(result)
    }

    #[tool(description = "Drain buffered stdout/stderr collected since the last call. Waits up to timeout_ms for new output.")]
    async fn read_output(&self, Parameters(params): Parameters<ReadOutputParams>) -> Result<CallToolResult, ErrorData> { ServerMethod::read_output(&self.processes, params).await.map_err(ErrorData::from) }

    #[tool(description = "Send text to a running process's stdin. A newline is appended automatically.")]
    async fn write_stdin(&self, Parameters(params): Parameters<WriteStdinParams>) -> Result<CallToolResult, ErrorData> {
        let (pid, text) = (params.pid, params.text.clone());
        let result = ServerMethod::write_stdin(&self.processes, params).await.map_err(ErrorData::from)?;
        self.log_action(format!("\u{2328}\u{FE0F} <b>write_stdin</b> pid={pid}: \"{}\"", html_escape(&truncate(&text, 80))));
        Ok(result)
    }

    #[tool(description = "Terminate a managed process and remove it from the list.")]
    async fn kill_process(&self, Parameters(params): Parameters<PidParam>) -> Result<CallToolResult, ErrorData> {
        let pid = params.pid;
        let result = ServerMethod::kill_process(&self.processes, params.pid).await.map_err(ErrorData::from)?;
        self.log_action(format!("\u{1F6D1} <b>kill_process</b> pid={pid}"));
        Ok(result)
    }

    #[tool(description = "List all managed processes with their IDs, command strings, and alive status.")]
    async fn list_processes(&self) -> Result<CallToolResult, ErrorData> { ServerMethod::list_processes(&self.processes).await.map_err(ErrorData::from) }

    #[tool(description = "Block until a spawned process prints a line containing the given substring, or the timeout expires. Scans all output since spawn, including lines already returned by read_output. Timeout is capped at 120s — call again to keep waiting.")]
    async fn wait_for_output(&self, Parameters(params): Parameters<WaitForOutputParams>) -> Result<CallToolResult, ErrorData> { ServerMethod::wait_for_output(&self.processes, params).await.map_err(ErrorData::from) }

    #[tool(description = "List the open desktop windows with their window IDs, owning PIDs, geometry, and titles. Pass a window_id to screenshot to capture just that window.")]
    async fn list_windows(&self) -> Result<CallToolResult, ErrorData> { ServerMethod::list_windows().await.map_err(ErrorData::from) }

    #[tool(description = "Click inside a window at coordinates relative to its top-left corner (same coordinates as a screenshot of that window). Raises the window first, so it steals focus. Screenshot the window afterwards to confirm the click landed.")]
    async fn click_window(&self, Parameters(params): Parameters<ClickParams>) -> Result<CallToolResult, ErrorData> {
        let (window_id, x, y, button) = (params.window_id.clone(), params.x, params.y, params.button);
        let result = ServerMethod::click_window(params).await.map_err(ErrorData::from)?;
        self.log_action(format!("\u{1F5B1} <b>click_window</b> {window_id} at ({x}, {y}) button={button}"));
        Ok(result)
    }

    #[tool(description = "Send a text message and/or a base64-encoded PNG photo to a pre-configured Telegram chat.")]
    async fn send_to_telegram(&self, Parameters(params): Parameters<SendToTelegramParams>) -> Result<CallToolResult, ErrorData> {
        let text_preview = params.text.as_deref().map(|t| truncate(t, 80));
        let result = ServerMethod::send_to_telegram(self.bot_token.as_deref(), self.chat_id.as_deref(), params).await.map_err(ErrorData::from)?;
        self.log_action(format!("\u{1F4E8} <b>send_to_telegram</b>{}", text_preview.map(|t| format!(": \"{}\"", html_escape(&t))).unwrap_or_default()));
        Ok(result)
    }

    #[tool(description = "Send a nicely formatted Telegram message (with the most recent screenshot attached, if one was taken) listing the actions (clicks, spawned processes, screenshots, stdin writes, kills) taken so far this session, then clear the log. Call this when wrapping up a task so the user sees a summary on their phone.")]
    async fn send_action_summary(&self) -> Result<CallToolResult, ErrorData> { ServerMethod::send_action_summary(&self.action_log, &self.last_screenshot, self.bot_token.as_deref(), self.chat_id.as_deref()).await.map_err(ErrorData::from) }
}

#[tool_handler]
#[rustfmt::skip]
impl ServerHandler for Server {
    fn get_info(&self) -> ServerInfo { ServerMethod::get_info() }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

fn truncate(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.trim_end_matches('\n').to_string()
    } else {
        let mut t: String = s.chars().take(max_chars).collect();
        t.push('\u{2026}');
        t
    }
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
