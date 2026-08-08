use rmcp::model::{CallToolResult, ContentBlock};

use crate::{Error, SendToTelegramParams};

use super::send_to_telegram_send;

pub async fn send_to_telegram(
    bot_token: Option<&str>,
    chat_id: Option<&str>,
    params: SendToTelegramParams,
) -> Result<CallToolResult, Error> {
    let (bot_token, chat_id) = match (bot_token, chat_id) {
        (Some(t), Some(c)) => (t, c),
        _ => {
            return Err(Error::Telegram(
                "TELEGRAM_BOT_TOKEN and TELEGRAM_CHAT_ID not configured".to_string(),
            ));
        }
    };

    let summary = send_to_telegram_send::send(
        bot_token,
        chat_id,
        params.text.as_deref(),
        params
            .photo_base64
            .as_deref()
            .map(|s| {
                base64::Engine::decode(&base64::engine::general_purpose::STANDARD, s)
                    .map_err(|e| Error::Telegram(format!("base64 decode: {e}")))
            })
            .transpose()?
            .as_deref(),
        None,
    )
    .await?;

    Ok(CallToolResult::success(vec![ContentBlock::text(summary)]))
}

#[cfg(test)]
mod tests {
    use crate::{Error, SendToTelegramParams};

    #[tokio::test]
    async fn test_usage_no_config() {
        let params = SendToTelegramParams {
            text: Some("hello".to_string()),
            photo_base64: None,
        };
        let result = super::send_to_telegram(None, None, params).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::Telegram(_)));
    }

    /// Sends a real message via the full send_to_telegram path.
    #[tokio::test]
    async fn test_usage_live_send() {
        let (token, chat_id) = crate::test_support::live_telegram_creds();
        let _guard = crate::test_support::live_test_lock().lock().await;

        let params = SendToTelegramParams {
            text: Some("deskctrl_mcp live test: send_to_telegram OK".to_string()),
            photo_base64: None,
        };
        let result = super::send_to_telegram(Some(&token), Some(&chat_id), params).await;
        let result = result.expect("live send_to_telegram");
        let text = format!("{result:?}");
        assert!(text.contains("text"), "expected a text-only summary, got {text}");
    }

    /// Sends a real photo (base64-encoded, as a real caller would) via the full
    /// send_to_telegram path and asserts the summary reports both photo and caption.
    #[tokio::test]
    async fn test_usage_live_send_photo() {
        let (token, chat_id) = crate::test_support::live_telegram_creds();
        let _guard = crate::test_support::live_test_lock().lock().await;

        // 1x1 red PNG, base64-encoded — a real caller sends exactly this shape.
        let photo_base64 = "iVBORw0KGgoAAAANSUhEUgAAAGQAAABkCAIAAAD/gAIDAAAA6klEQVR4nO3QQQ3AIADAQEAX/oMsLKwvsuROQdN59h58s14H/IlZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVnBBWW/AghGbV2gAAAAAElFTkSuQmCC".to_string();
        let params = SendToTelegramParams {
            text: Some("deskctrl_mcp live test: send_to_telegram photo OK".to_string()),
            photo_base64: Some(photo_base64),
        };
        let result = super::send_to_telegram(Some(&token), Some(&chat_id), params).await;
        let result = result.expect("live send_to_telegram photo");
        let text = format!("{result:?}");
        assert!(
            text.contains("photo") && text.contains("caption"),
            "expected photo+caption summary, got {text}"
        );
    }
}
