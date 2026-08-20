use std::{fs, process::Command, sync::Arc, time::Duration};

use tokio::sync::Mutex;

use crate::{Error, Recording};

use super::stopped_recording::StoppedRecording;

pub async fn stop(recording: &Arc<Mutex<Option<Recording>>>) -> Result<StoppedRecording, Error> {
    let rec = {
        let mut guard = recording.lock().await;
        guard.take().ok_or_else(|| {
            Error::Window("not recording \u{2014} call record_video to start".to_string())
        })?
    };

    rec.keep_awake.abort();

    let target = rec.target;
    let path = rec.path;
    let mut child = rec.child;
    let pid = child.id();

    let _ = Command::new("kill")
        .args(["-INT", &pid.to_string()])
        .status();

    let wait_task = tokio::task::spawn_blocking(move || child.wait());
    match tokio::time::timeout(Duration::from_secs(10), wait_task).await {
        Ok(Ok(status)) => {
            tracing::info!(target: "deskctrl_mcp::recording", "ffmpeg exited {status:?}");
        }
        Ok(Err(e)) => {
            tracing::warn!(target: "deskctrl_mcp::recording", "ffmpeg wait task failed: {e}");
        }
        Err(_elapsed) => {
            tracing::warn!(target: "deskctrl_mcp::recording", "ffmpeg did not exit on SIGINT; sending SIGKILL");
            let _ = Command::new("kill").args(["-9", &pid.to_string()]).status();
        }
    }

    let duration_secs = rec.started.elapsed().unwrap_or_default().as_secs();
    let size_bytes = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);

    Ok(StoppedRecording {
        path,
        duration_secs,
        size_bytes,
        target,
    })
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::Error;

    #[tokio::test]
    async fn test_usage() {
        let recording = Arc::new(tokio::sync::Mutex::new(None));
        let result = super::stop(&recording).await;
        assert!(matches!(result, Err(Error::Window(_))));
    }
}
