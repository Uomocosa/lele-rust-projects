use crate::Error;

/// Sends an MP4 video (optionally with an HTML caption) via `sendVideo`, returning Telegram's
/// `Message` object. Mirrors [`send_raw`] for videos so a caller can assert what was sent.
pub async fn send_video(
    bot_token: &str,
    chat_id: &str,
    mp4: &[u8],
    caption: Option<&str>,
) -> Result<serde_json::Value, Error> {
    let client = reqwest::Client::new();
    let url = format!("https://api.telegram.org/bot{bot_token}/sendVideo");
    let part = reqwest::multipart::Part::bytes(mp4.to_vec())
        .file_name("session.mp4")
        .mime_str("video/mp4")
        .map_err(|e| Error::Telegram(format!("mime: {e}")))?;
    let mut form = reqwest::multipart::Form::new()
        .text("chat_id", chat_id.to_string())
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
        .await
        .map_err(|e| Error::Telegram(format!("sendVideo: {e}")))?;
    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| Error::Telegram(format!("sendVideo parse: {e}")))?;
    if body["ok"] != serde_json::Value::Bool(true) {
        return Err(Error::Telegram(format!("sendVideo failed: {body}")));
    }
    Ok(body["result"].clone())
}

#[cfg(test)]
mod tests {
    use crate::Error;

    #[tokio::test]
    async fn test_usage() {
        let result = super::send_video("bad_token", "123", &[], Some("caption")).await;
        assert!(matches!(result, Err(Error::Telegram(_))));
    }

    /// Generates a real 1-second test video with ffmpeg and asserts the sendVideo path uploads
    /// it, echoing a `video` object back in Telegram's response.
    #[tokio::test]
    async fn test_usage_live_send_video() {
        let (token, chat_id) = crate::test_support::live_telegram_creds();
        let _guard = crate::test_support::live_test_lock().lock().await;

        let path = format!("/tmp/deskctrl-mcp-send-video-{}.mp4", std::process::id());
        let ok = std::process::Command::new("ffmpeg")
            .args([
                "-y",
                "-f",
                "lavfi",
                "-i",
                "testsrc=duration=1:size=64x64:rate=10",
                "-c:v",
                "libx264",
                "-pix_fmt",
                "yuv420p",
                &path,
            ])
            .output()
            .expect("ffmpeg present for live send_video test");
        assert!(ok.status.success(), "ffmpeg failed: {ok:?}");
        let mp4 = std::fs::read(&path).expect("reading generated mp4");
        let _ = std::fs::remove_file(&path);

        let caption = "deskctrl_mcp live test: send_video OK";
        let sent = super::send_video(&token, &chat_id, &mp4, Some(caption))
            .await
            .expect("live sendVideo");
        assert_eq!(sent["caption"].as_str(), Some(caption));
        assert!(
            sent["video"].is_object(),
            "expected video object, got {sent}"
        );
    }
}
