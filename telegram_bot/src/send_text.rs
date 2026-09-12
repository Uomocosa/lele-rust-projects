use crate::Creds;
use crate::Error;
use crate::message_id;

/// # Errors
/// Returns an error if the text is empty, the request fails,
/// or Telegram answers with a non-success status or `ok:false`.
pub fn send_text(creds: &Creds, text: &str) -> Result<String, Error> {
    if text.is_empty() {
        return Err(Error::EmptyPayload { op: "send_text" });
    }
    let token = creds.token.as_str();
    let chat_id = creds.chat_id.as_str();
    let client = reqwest::blocking::Client::new();
    let url = format!("https://api.telegram.org/bot{token}/sendMessage");
    let response = client
        .post(&url)
        .json(&serde_json::json!({
            "chat_id": chat_id,
            "text": text
        }))
        .send()
        .map_err(|err| Error::Request {
            op: "sendMessage",
            message: err.to_string(),
        })?;
    let status = response.status();
    let status_text = status.to_string();
    let body = response.text().unwrap_or_default();
    if !status.is_success() {
        let snippet: String = body.chars().take(500).collect();
        return Err(Error::Rejected {
            op: "sendMessage",
            status: status_text,
            snippet,
        });
    }
    message_id::message_id("sendMessage", &status_text, &body)
}

#[cfg(test)]
mod tests {
    use super::send_text;
    use crate::Creds;

    #[test]
    fn test_usage() {
        let creds = Creds {
            token: "token".to_string(),
            chat_id: "chat".to_string(),
        };
        assert!(send_text(&creds, "").is_err());
        let _ = send_text;
    }
}
