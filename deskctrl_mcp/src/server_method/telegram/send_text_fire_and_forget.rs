use super::send;

pub fn send_text_fire_and_forget(bot_token: String, chat_id: String, html: String) {
    tokio::spawn(async move {
        match send::send(&bot_token, &chat_id, Some(&html), None, Some("HTML")).await {
            Ok(summary) => {
                tracing::info!(target: "deskctrl_mcp::telegram", "{summary}");
            }
            Err(e) => {
                tracing::warn!(target: "deskctrl_mcp::telegram", "telegram text send failed: {e}");
            }
        }
    });
}

// no test_usage necessary
