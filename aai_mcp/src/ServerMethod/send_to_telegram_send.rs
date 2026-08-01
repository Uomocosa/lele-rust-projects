use crate::Error;

pub async fn send(
    bot_token: &str,
    chat_id: &str,
    text: Option<&str>,
    photo_png: Option<&[u8]>,
) -> Result<String, Error> {
    let client = reqwest::Client::new();
    let mut parts = Vec::new();

    if let Some(t) = text {
        let url = format!("https://api.telegram.org/bot{bot_token}/sendMessage");
        let resp = client
            .post(&url)
            .json(&serde_json::json!({"chat_id": chat_id, "text": t}))
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
        parts.push("text".to_string());
    }

    if let Some(png) = photo_png {
        let url = format!("https://api.telegram.org/bot{bot_token}/sendPhoto");
        let part = reqwest::multipart::Part::bytes(png.to_vec())
            .file_name("screenshot.png")
            .mime_str("image/png")
            .map_err(|e| Error::Telegram(format!("mime: {e}")))?;
        let form = reqwest::multipart::Form::new()
            .text("chat_id", chat_id.to_string())
            .part("photo", part);
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
        parts.push("photo".to_string());
    }

    Ok(format!("sent to Telegram: {}", parts.join(" + ")))
}

pub fn send_photo_fire_and_forget(bot_token: String, chat_id: String, png: Vec<u8>) {
    tokio::spawn(async move {
        match send(&bot_token, &chat_id, None, Some(&png)).await {
            Ok(summary) => {
                tracing::info!(target: "aai_mcp::telegram", "{summary}");
            }
            Err(e) => {
                tracing::warn!(target: "aai_mcp::telegram", "telegram photo send failed: {e}");
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use crate::Error;

    #[tokio::test]
    async fn test_usage_bad_token() {
        let result = super::send("bad_token", "123", Some("hello"), None).await;
        assert!(matches!(result, Err(Error::Telegram(_))));
    }
}
