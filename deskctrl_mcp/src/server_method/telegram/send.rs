use crate::Error;

use super::send_raw;

/// When both `text` and `photo_png` are given, the text goes out as the photo's caption in a
/// single `sendPhoto` call (matches Telegram's own UX for "here's what happened, with a
/// picture") rather than two separate messages.
pub async fn send(
    bot_token: &str,
    chat_id: &str,
    text: Option<&str>,
    photo_png: Option<&[u8]>,
    parse_mode: Option<&str>,
) -> Result<String, Error> {
    let sent = send_raw::send_raw(bot_token, chat_id, text, photo_png, parse_mode).await?;
    let mut parts = Vec::new();
    if sent["photo"].is_array() {
        parts.push("photo");
    }
    if sent["caption"].is_string() {
        parts.push("caption");
    } else if sent["text"].is_string() {
        parts.push("text");
    }
    Ok(format!("sent to Telegram: {}", parts.join(" + ")))
}

#[cfg(test)]
mod tests {
    use crate::Error;

    #[tokio::test]
    async fn test_usage() {
        let result = super::send("bad_token", "123", Some("hello"), None, None).await;
        assert!(matches!(result, Err(Error::Telegram(_))));
    }
}
