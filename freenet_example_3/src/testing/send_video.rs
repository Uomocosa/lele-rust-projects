use crate::testing;

#[allow(clippy::expect_used)]
pub fn send_video(creds: &testing::Creds, bytes: &[u8], caption: &str) {
    let client = reqwest::blocking::Client::new();
    let url = format!("https://api.telegram.org/bot{}/sendVideo", creds.token);
    let form = reqwest::blocking::multipart::Form::new()
        .text("chat_id", creds.chat_id.clone())
        .text("caption", caption.to_string())
        .part(
            "video",
            reqwest::blocking::multipart::Part::bytes(bytes.to_vec())
                .file_name("clip.mp4")
                .mime_str("video/mp4")
                .unwrap_or_else(|_| {
                    reqwest::blocking::multipart::Part::bytes(bytes.to_vec()).file_name("clip.mp4")
                }),
        );
    match client.post(&url).multipart(form).send() {
        Ok(resp) => {
            if !resp.status().is_success() {
                let status = resp.status();
                let body = resp.text().unwrap_or_default();
                let snippet: String = body.chars().take(500).collect();
                eprintln!("telegram sendVideo failed: status={status} body={snippet}");
            }
        }
        Err(err) => {
            eprintln!("telegram sendVideo request failed: {err}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::send_video;

    #[test]
    fn test_usage() {
        let _ = send_video;
    }
}
