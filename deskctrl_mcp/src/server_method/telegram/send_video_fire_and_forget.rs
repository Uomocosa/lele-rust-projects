use super::send_video;

pub fn send_video_fire_and_forget(
    bot_token: String,
    chat_id: String,
    mp4: Vec<u8>,
    caption: String,
) {
    tokio::spawn(async move {
        match send_video::send_video(&bot_token, &chat_id, &mp4, Some(&caption)).await {
            Ok(_) => {
                tracing::info!(target: "deskctrl_mcp::telegram", "sent video to Telegram");
            }
            Err(e) => {
                tracing::warn!(target: "deskctrl_mcp::telegram", "telegram video send failed: {e}");
            }
        }
    });
}

// no test_usage necessary
