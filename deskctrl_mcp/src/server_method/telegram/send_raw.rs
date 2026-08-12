use crate::Error;

/// Same as [`send`], but returns Telegram's own `Message` object (the `result` field of a
/// successful `sendMessage`/`sendPhoto` response) instead of a display string. Since bots
/// cannot read arbitrary chat history back from the Bot API, this response — which Telegram
/// echoes back with exactly what it stored (message_id, text/caption, photo file_ids, ...) — is
/// the authoritative way to assert what was actually sent, without a separate "read" round trip.
pub async fn send_raw(
    bot_token: &str,
    chat_id: &str,
    text: Option<&str>,
    photo_png: Option<&[u8]>,
    parse_mode: Option<&str>,
) -> Result<serde_json::Value, Error> {
    let client = reqwest::Client::new();

    if let Some(png) = photo_png {
        let url = format!("https://api.telegram.org/bot{bot_token}/sendPhoto");
        let part = reqwest::multipart::Part::bytes(png.to_vec())
            .file_name("screenshot.png")
            .mime_str("image/png")
            .map_err(|e| Error::Telegram(format!("mime: {e}")))?;
        let mut form = reqwest::multipart::Form::new()
            .text("chat_id", chat_id.to_string())
            .part("photo", part);
        if let Some(caption) = text {
            form = form.text("caption", caption.to_string());
            if let Some(mode) = parse_mode {
                form = form.text("parse_mode", mode.to_string());
            }
        }
        let resp = client
            .post(&url)
            .multipart(form)
            .send()
            .await
            .map_err(|e| Error::Telegram(format!("sendPhoto: {e}")))?;
        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| Error::Telegram(format!("sendPhoto parse: {e}")))?;
        if body["ok"] != serde_json::Value::Bool(true) {
            return Err(Error::Telegram(format!("sendPhoto failed: {body}")));
        }
        return Ok(body["result"].clone());
    }

    if let Some(t) = text {
        let url = format!("https://api.telegram.org/bot{bot_token}/sendMessage");
        let mut body = serde_json::json!({"chat_id": chat_id, "text": t});
        if let Some(mode) = parse_mode {
            body["parse_mode"] = serde_json::Value::String(mode.to_string());
        }
        let resp = client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Telegram(format!("sendMessage: {e}")))?;
        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| Error::Telegram(format!("sendMessage parse: {e}")))?;
        if body["ok"] != serde_json::Value::Bool(true) {
            return Err(Error::Telegram(format!("sendMessage failed: {body}")));
        }
        return Ok(body["result"].clone());
    }

    Err(Error::Telegram(
        "send: neither text nor photo_png given".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use crate::Error;
    use crate::test_support;

    // needed helper: a real, minimal 1x1 red PNG so live photo tests exercise actual image
    // upload/decoding on Telegram's side rather than a synthetic byte string.
    fn tiny_png() -> Vec<u8> {
        use base64::Engine;
        base64::engine::general_purpose::STANDARD
            .decode("iVBORw0KGgoAAAANSUhEUgAAAGQAAABkCAIAAAD/gAIDAAAA6klEQVR4nO3QQQ3AIADAQEAX/oMsLKwvsuROQdN59h58s14H/IlZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVmBWYFZgVnBBWW/AghGbV2gAAAAAElFTkSuQmCC")
            .unwrap()
    }

    #[tokio::test]
    async fn test_usage() {
        let result = super::send_raw("bad_token", "123", Some("hello"), None, None).await;
        assert!(matches!(result, Err(Error::Telegram(_))));
    }

    /// Sends a real message and asserts on Telegram's own response — the returned Message
    /// object's `text` field is Telegram's authoritative confirmation of what it stored, not
    /// just "the HTTP call didn't error".
    #[tokio::test]
    async fn test_usage_live_send() {
        let (token, chat_id) = test_support::live_telegram_creds();
        let _guard = test_support::live_test_lock().lock().await;

        let text = "deskctrl_mcp live test: send_raw OK";
        let sent = super::send_raw(&token, &chat_id, Some(text), None, None)
            .await
            .expect("live sendMessage");
        assert_eq!(sent["text"].as_str(), Some(text));
        assert!(sent["message_id"].is_number());
    }

    /// Sends a real photo with a caption via the single sendPhoto+caption path and asserts
    /// Telegram's response actually contains a photo array and the matching caption text.
    #[tokio::test]
    async fn test_usage_live_send_photo() {
        let (token, chat_id) = test_support::live_telegram_creds();
        let _guard = test_support::live_test_lock().lock().await;

        let caption = "deskctrl_mcp live test: send_raw photo OK";
        let sent = super::send_raw(&token, &chat_id, Some(caption), Some(&tiny_png()), None)
            .await
            .expect("live sendPhoto");
        assert_eq!(sent["caption"].as_str(), Some(caption));
        let photos = sent["photo"].as_array().expect("photo array in response");
        assert!(
            !photos.is_empty(),
            "expected at least one photo size, got {sent}"
        );
    }
}
