use crate::testing;

pub fn send_text(creds: &testing::Creds, text: &str) {
    let token = creds.token.clone();
    let chat_id = creds.chat_id.clone();
    let text = text.to_string();
    let handle = std::thread::spawn(move || {
        let client = reqwest::blocking::Client::new();
        let url = format!("https://api.telegram.org/bot{token}/sendMessage");
        match client
            .post(&url)
            .json(&serde_json::json!({
                "chat_id": chat_id,
                "text": text
            }))
            .send()
        {
            Ok(resp) => {
                if !resp.status().is_success() {
                    let status = resp.status();
                    let body = resp.text().unwrap_or_default();
                    let snippet: String = body.chars().take(500).collect();
                    eprintln!("telegram sendMessage failed: status={status} body={snippet}");
                }
            }
            Err(err) => {
                eprintln!("telegram sendMessage request failed: {err}");
            }
        }
    });
    let _ = handle.join();
}

#[cfg(test)]
mod tests {
    use super::send_text;

    #[test]
    fn test_usage() {
        let _ = send_text;
    }
}
