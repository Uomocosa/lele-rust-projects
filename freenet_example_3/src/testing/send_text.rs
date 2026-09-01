use crate::testing;

pub fn send_text(creds: &testing::Creds, text: &str) {
    let client = reqwest::blocking::Client::new();
    let url = format!("https://api.telegram.org/bot{}/sendMessage", creds.token);
    match client
        .post(&url)
        .json(&serde_json::json!({
            "chat_id": creds.chat_id,
            "text": text,
            "parse_mode": "Markdown"
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
}

#[cfg(test)]
mod tests {
    use super::send_text;

    #[test]
    fn test_usage() {
        let _ = send_text;
    }
}
