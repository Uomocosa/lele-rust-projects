use super::send;

pub fn send_photo_caption_fire_and_forget(
    bot_token: String,
    chat_id: String,
    png: Vec<u8>,
    caption: Option<String>,
) {
    tokio::spawn(async move {
        match send::send(
            &bot_token,
            &chat_id,
            caption.as_deref(),
            Some(&png),
            Some("HTML"),
        )
        .await
        {
            Ok(summary) => {
                tracing::info!(target: "deskctrl_mcp::telegram", "{summary}");
            }
            Err(e) => {
                tracing::warn!(target: "deskctrl_mcp::telegram", "telegram photo send failed: {e}");
            }
        }
    });
}

// no test_usage necessary
