use crate::Error;
use crate::creds;

pub fn send_text(creds: &creds::Creds, text: &str) -> Result<String, Error> {
    let client = reqwest::blocking::Client::new();
    let url = format!("https://api.telegram.org/bot{}/sendMessage", creds.token);
    let resp = client
        .post(&url)
        .form(&[
            ("chat_id", creds.chat_id.clone()),
            ("text", text.to_string()),
            ("parse_mode", "HTML".to_string()),
        ])
        .send()
        .map_err(|e| Error::Telegram(format!("sendMessage: {e}")))?;
    let body: serde_json::Value = resp
        .json()
        .map_err(|e| Error::Telegram(format!("sendMessage parse: {e}")))?;
    if body["ok"] != serde_json::Value::Bool(true) {
        return Err(Error::Telegram(format!("sendMessage failed: {body}")));
    }
    Ok("report sent to Telegram".to_string())
}

// no test_usage necessary
