use crate::Error;
use crate::creds;

pub fn send_video(
    creds: &creds::Creds,
    mp4: &[u8],
    caption: Option<&str>,
) -> Result<String, Error> {
    let client = reqwest::blocking::Client::new();
    let url = format!("https://api.telegram.org/bot{}/sendVideo", creds.token);
    let part = reqwest::blocking::multipart::Part::bytes(mp4.to_vec())
        .file_name("session.mp4")
        .mime_str("video/mp4")
        .map_err(|e| Error::Telegram(format!("mime: {e}")))?;
    let mut form = reqwest::blocking::multipart::Form::new()
        .text("chat_id", creds.chat_id.clone())
        .part("video", part);
    if let Some(caption) = caption {
        form = form
            .text("caption", caption.to_string())
            .text("parse_mode", "HTML");
    }
    let resp = client
        .post(&url)
        .multipart(form)
        .send()
        .map_err(|e| Error::Telegram(format!("sendVideo: {e}")))?;
    let body: serde_json::Value = resp
        .json()
        .map_err(|e| Error::Telegram(format!("sendVideo parse: {e}")))?;
    if body["ok"] != serde_json::Value::Bool(true) {
        return Err(Error::Telegram(format!("sendVideo failed: {body}")));
    }
    Ok("video sent to Telegram".to_string())
}

// no test_usage necessary
