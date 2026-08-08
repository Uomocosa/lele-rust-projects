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
}
