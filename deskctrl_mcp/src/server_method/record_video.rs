use std::{fs, sync::Arc};

use rmcp::model::{CallToolResult, ContentBlock};
use tokio::sync::Mutex;

use crate::{Error, RecordVideoParams, Recording, recording_method};

const TELEGRAM_VIDEO_CAP: u64 = 50 * 1024 * 1024;

pub async fn record_video(
    recording: &Arc<Mutex<Option<Recording>>>,
    bot_token: Option<&str>,
    chat_id: Option<&str>,
    artifacts_dir: Option<&str>,
    params: RecordVideoParams,
) -> Result<CallToolResult, Error> {
    if params.stop {
        stop_and_send(recording, bot_token, chat_id, params).await
    } else {
        let desc = recording_method::start(
            recording,
            artifacts_dir,
            params.window_id.as_deref(),
            params.pid,
            params.title.as_deref(),
        )
        .await?;
        Ok(CallToolResult::success(vec![ContentBlock::text(desc)]))
    }
}

async fn stop_and_send(
    recording: &Arc<Mutex<Option<Recording>>>,
    bot_token: Option<&str>,
    chat_id: Option<&str>,
    params: RecordVideoParams,
) -> Result<CallToolResult, Error> {
    let stopped = recording_method::stop(recording).await?;
    let caption = params
        .summary
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| stopped.caption());

    let mut sent_note = "not sent (no Telegram config or file too large)".to_string();
    if params.send_to_telegram
        && let (Some(token), Some(cid)) = (bot_token, chat_id)
    {
        let mp4 = fs::read(&stopped.path)
            .map_err(|e| Error::Screenshot(format!("read recording: {e}")))?;
        if mp4.len() as u64 <= TELEGRAM_VIDEO_CAP {
            let result = super::telegram::send_video(token, cid, &mp4, Some(&caption)).await?;
            if result["video"].is_object() {
                sent_note = "sent to Telegram".to_string();
            }
        } else {
            tracing::warn!(target: "deskctrl_mcp::recording", "recording too large to send ({} bytes)", mp4.len());
        }
    }

    let mb = stopped.size_bytes as f64 / (1024.0 * 1024.0);
    Ok(CallToolResult::success(vec![ContentBlock::text(format!(
        "recording stopped: {} \u{2014} {}s, {:.1} MB; {sent_note}",
        stopped.target, stopped.duration_secs, mb
    ))]))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{Error, RecordVideoParams};

    #[tokio::test]
    async fn test_usage() {
        let recording = Arc::new(tokio::sync::Mutex::new(None));
        let params = RecordVideoParams {
            window_id: None,
            pid: None,
            title: None,
            stop: true,
            summary: None,
            send_to_telegram: false,
        };
        let result = super::record_video(&recording, None, None, None, params).await;
        assert!(matches!(result, Err(Error::Window(_))));
    }

    /// Records ~1s of the live display (no Telegram, so nothing is sent) and asserts a clean
    /// start → stop round trip produces a non-empty MP4.
    #[tokio::test]
    async fn test_usage_live_display() {
        use std::fs;

        crate::test_support::assert_live_display();
        let _guard = crate::test_support::live_test_lock().lock().await;

        let recording = Arc::new(tokio::sync::Mutex::new(None));
        let start = RecordVideoParams {
            window_id: None,
            pid: None,
            title: None,
            stop: false,
            summary: None,
            send_to_telegram: false,
        };
        let start_result = super::record_video(&recording, None, None, None, start).await;
        let start_text = format!("{:?}", start_result);
        assert!(start_text.contains("recording started"), "{start_text}");

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        let stop = RecordVideoParams {
            window_id: None,
            pid: None,
            title: None,
            stop: true,
            summary: None,
            send_to_telegram: false,
        };
        let stop_result = super::record_video(&recording, None, None, None, stop).await;
        let stop_text = format!("{stop_result:?}");
        assert!(stop_text.contains("recording stopped"), "{stop_text}");

        assert!(
            recording.lock().await.is_none(),
            "recording should be cleared after stop"
        );
        let dir = std::env::temp_dir();
        let files = fs::read_dir(dir).expect("reading temp dir");
        assert!(
            files.filter_map(|e| e.ok()).any(|e| {
                let path = e.path();
                path.extension().map(|x| x == "mp4").unwrap_or(false)
                    && path
                        .file_name()
                        .map(|n| n.to_string_lossy().starts_with("deskctrl-mcp-recording-"))
                        .unwrap_or(false)
            }),
            "a recording mp4 should exist after start/stop"
        );
    }
}
