use std::sync::Mutex;

use rmcp::model::{CallToolResult, ContentBlock};

use crate::Error;

use super::send_to_telegram_send;

pub async fn send_action_summary(
    action_log: &Mutex<Vec<String>>,
    last_screenshot: &Mutex<Option<Vec<u8>>>,
    bot_token: Option<&str>,
    chat_id: Option<&str>,
) -> Result<CallToolResult, Error> {
    let (bot_token, chat_id) = match (bot_token, chat_id) {
        (Some(t), Some(c)) => (t, c),
        _ => {
            return Err(Error::Telegram(
                "TELEGRAM_BOT_TOKEN and TELEGRAM_CHAT_ID not configured".to_string(),
            ));
        }
    };

    let entries = {
        let log = action_log.lock().unwrap();
        log.clone()
    };

    if entries.is_empty() {
        return Ok(CallToolResult::success(vec![ContentBlock::text(
            "no actions logged since the last summary; nothing sent",
        )]));
    }

    let header = format!(
        "\u{1F4CB} <b>Session summary</b> \u{2014} {} action{}",
        entries.len(),
        if entries.len() == 1 { "" } else { "s" }
    );
    let message = format!("{header}\n\n{}", entries.join("\n"));

    // Attach the most recent screenshot taken this session, if any, as the photo's caption
    // rather than a separate message — one glance shows both what happened and what it looked
    // like.
    let photo = last_screenshot.lock().unwrap().take();

    let summary = send_to_telegram_send::send(
        bot_token,
        chat_id,
        Some(&message),
        photo.as_deref(),
        Some("HTML"),
    )
    .await?;

    action_log.lock().unwrap().clear();

    Ok(CallToolResult::success(vec![ContentBlock::text(summary)]))
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use crate::Error;

    #[tokio::test]
    async fn test_usage_no_config() {
        let log = Mutex::new(vec!["click".to_string()]);
        let shot = Mutex::new(None);
        let result = super::send_action_summary(&log, &shot, None, None).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::Telegram(_)));
    }

    #[tokio::test]
    async fn test_usage_empty_log_short_circuits() {
        let log = Mutex::new(Vec::new());
        let shot = Mutex::new(None);
        let result = super::send_action_summary(&log, &shot, Some("bad_token"), Some("123")).await;
        assert!(result.is_ok());
        let text = format!("{:?}", result.unwrap());
        assert!(text.contains("nothing sent"));
    }

    /// Sends a real summary with no screenshot attached (text-only path).
    #[tokio::test]
    async fn test_usage_live_send() {
        let (token, chat_id) = crate::test_support::live_telegram_creds();
        let _guard = crate::test_support::live_test_lock().lock().await;

        let log = Mutex::new(vec!["deskctrl_mcp live test: send_action_summary".to_string()]);
        let shot = Mutex::new(None);
        let result = super::send_action_summary(&log, &shot, Some(&token), Some(&chat_id)).await;
        let result = result.expect("live send_action_summary");
        let text = format!("{result:?}");
        assert!(text.contains("text") && !text.contains("photo"), "expected text-only summary, got {text}");
        assert!(log.lock().unwrap().is_empty(), "log should clear after send");
    }

    /// Sends a real summary WITH a "screenshot" attached (a real 1x1 PNG), asserting the
    /// resulting Telegram message actually carries a photo — this is the path a real session
    /// hits after calling `screenshot` before wrapping up with `send_action_summary`.
    #[tokio::test]
    async fn test_usage_live_send_with_screenshot() {
        let (token, chat_id) = crate::test_support::live_telegram_creds();
        let _guard = crate::test_support::live_test_lock().lock().await;

        use base64::Engine;
        let tiny_png = base64::engine::general_purpose::STANDARD
            .decode("iVBORw0KGgoAAAANSUhEUgAAAGQAAABkCAIAAAD/gAIDAAAA6klEQVR4nO3QQQ3AIADAQEAX/oMsLKwvsuROQdN59h58s14H/IlZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVnBBWW/AghGbV2gAAAAAElFTkSuQmCC")
            .unwrap();

        let log = Mutex::new(vec![
            "deskctrl_mcp live test: send_action_summary with screenshot".to_string(),
        ]);
        let shot = Mutex::new(Some(tiny_png));
        let result = super::send_action_summary(&log, &shot, Some(&token), Some(&chat_id)).await;
        let result = result.expect("live send_action_summary with screenshot");
        let text = format!("{result:?}");
        assert!(text.contains("photo"), "expected a photo summary, got {text}");
        assert!(shot.lock().unwrap().is_none(), "screenshot should be taken/cleared after send");
    }
}
