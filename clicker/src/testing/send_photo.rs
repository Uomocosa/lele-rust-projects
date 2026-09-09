use crate::testing;

/// # Errors
/// Returns an error if the photo bytes are empty, the upload thread panics,
/// the request fails, or Telegram answers with a non-success status.
pub fn send_photo(creds: &testing::Creds, bytes: &[u8], caption: &str) -> Result<String, String> {
    if bytes.is_empty() {
        return Err("telegram send_photo: empty photo bytes".to_string());
    }
    let token = creds.token.clone();
    let chat_id = creds.chat_id.clone();
    let caption = caption.to_string();
    let bytes = bytes.to_vec();
    let handle =
        std::thread::spawn(move || send_photo_blocking(&token, &chat_id, &bytes, &caption));
    handle
        .join()
        .unwrap_or_else(|_| Err("telegram send_photo: upload thread panicked".to_string()))
}

fn send_photo_blocking(
    token: &str,
    chat_id: &str,
    bytes: &[u8],
    caption: &str,
) -> Result<String, String> {
    let client = reqwest::blocking::Client::new();
    let url = format!("https://api.telegram.org/bot{token}/sendPhoto");
    let form = reqwest::blocking::multipart::Form::new()
        .text("chat_id", chat_id.to_string())
        .text("caption", caption.to_string())
        .part(
            "photo",
            reqwest::blocking::multipart::Part::bytes(bytes.to_vec())
                .file_name("preview.png")
                .mime_str("image/png")
                .unwrap_or_else(|_| {
                    reqwest::blocking::multipart::Part::bytes(bytes.to_vec())
                        .file_name("preview.png")
                }),
        );
    let response = client
        .post(&url)
        .multipart(form)
        .send()
        .map_err(|err| format!("telegram sendPhoto request failed: {err}"))?;
    let status = response.status();
    let body = response.text().unwrap_or_default();
    if !status.is_success() {
        let snippet: String = body.chars().take(500).collect();
        return Err(format!(
            "telegram sendPhoto failed: status={status} body={snippet}"
        ));
    }
    Ok(message_id(&body))
}

// needed helper: scans the raw JSON without a schema type
fn message_id(body: &str) -> String {
    let Some(start) = body.find("\"message_id\"") else {
        return "ok".to_string();
    };
    let Some(rest) = body.get(start.saturating_add(12)..) else {
        return "ok".to_string();
    };
    let digits: String = rest
        .chars()
        .skip_while(|c| !c.is_ascii_digit())
        .take_while(char::is_ascii_digit)
        .collect();
    if digits.is_empty() {
        "ok".to_string()
    } else {
        digits
    }
}

#[cfg(test)]
mod tests {
    use super::{message_id, send_photo};
    use crate::testing;

    #[test]
    fn test_usage() {
        assert_eq!(
            message_id(r#"{"ok":true,"result":{"message_id":42}}"#),
            "42"
        );
        assert_eq!(message_id("garbage"), "ok");
        let creds = testing::Creds {
            token: "t".to_string(),
            chat_id: "c".to_string(),
        };
        assert!(send_photo(&creds, &[], "caption").is_err());
        let _ = send_photo;
    }
}
