use crate::Creds;
use crate::Error;
use crate::message_id;

/// # Errors
/// Returns an error if the video bytes are empty, the request fails,
/// or Telegram answers with a non-success status or `ok:false`.
pub fn send_video(creds: &Creds, bytes: &[u8], caption: &str) -> Result<String, Error> {
    if bytes.is_empty() {
        return Err(Error::EmptyPayload { op: "send_video" });
    }
    let token = creds.token.as_str();
    let chat_id = creds.chat_id.as_str();
    let client = reqwest::blocking::Client::new();
    let url = format!("https://api.telegram.org/bot{token}/sendVideo");
    let form = reqwest::blocking::multipart::Form::new()
        .text("chat_id", chat_id.to_owned())
        .text("caption", caption.to_owned())
        .part(
            "video",
            reqwest::blocking::multipart::Part::bytes(bytes.to_vec())
                .file_name("clip.mp4")
                .mime_str("video/mp4")
                .unwrap_or_else(|_| {
                    reqwest::blocking::multipart::Part::bytes(bytes.to_vec()).file_name("clip.mp4")
                }),
        );
    let response = client
        .post(&url)
        .multipart(form)
        .send()
        .map_err(|err| Error::Request {
            op: "sendVideo",
            message: err.to_string(),
        })?;
    let status = response.status();
    let status_text = status.to_string();
    let body = response.text().unwrap_or_default();
    if !status.is_success() {
        let snippet: String = body.chars().take(500).collect();
        return Err(Error::Rejected {
            op: "sendVideo",
            status: status_text,
            snippet,
        });
    }
    message_id::message_id("sendVideo", &status_text, &body)
}

#[cfg(test)]
mod tests {
    use super::send_video;
    use crate::Creds;

    #[test]
    fn test_usage() {
        let creds = Creds {
            token: "token".to_string(),
            chat_id: "chat".to_string(),
        };
        assert!(send_video(&creds, &[], "caption").is_err());
        let _ = send_video;
    }
}
