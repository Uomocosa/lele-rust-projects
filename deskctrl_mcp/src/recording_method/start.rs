use std::{
    process::{Child, Command, Stdio},
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use tokio::sync::Mutex;

use crate::{Error, Recording, WindowInfo, window_info_method};

const FRAME_RATE: &str = "15";
const CRF: &str = "28";
const DEFAULT_MAX_SECS: u64 = 600;

struct Capture {
    pub size: String,
    pub offset: String,
    pub desc: String,
}

pub async fn start(
    recording: &Arc<Mutex<Option<Recording>>>,
    artifacts_dir: Option<&str>,
    window_id: Option<&str>,
    pid: Option<u32>,
    title: Option<&str>,
) -> Result<String, Error> {
    check_ffmpeg()?;

    let mut guard = recording.lock().await;
    if guard.is_some() {
        return Err(Error::Window(
            "already recording \u{2014} call record_video with stop:true first".to_string(),
        ));
    }

    let capture = resolve_capture(window_id, pid, title)?;
    let path = video_path(artifacts_dir);
    let max_secs = max_secs();
    let child = spawn_ffmpeg(&capture, &path, max_secs)?;

    *guard = Some(Recording {
        path: path.clone(),
        child,
        started: SystemTime::now(),
        target: capture.desc.clone(),
    });

    Ok(format!(
        "recording started: {} \u{2192} {} (max {max_secs}s; will stop + send to Telegram on stop)",
        capture.desc, path
    ))
}

fn check_ffmpeg() -> Result<(), Error> {
    match Command::new("ffmpeg").arg("-version").output() {
        Ok(out) if out.status.success() => Ok(()),
        _ => Err(Error::Ffmpeg("ffmpeg not found on PATH".to_string())),
    }
}

// needed helper:
fn resolve_capture(
    window_id: Option<&str>,
    pid: Option<u32>,
    title: Option<&str>,
) -> Result<Capture, Error> {
    if window_id.is_none() && pid.is_none() && title.is_none() {
        let size = screen_size()?;
        return Ok(Capture {
            size,
            offset: "+0,0".to_string(),
            desc: "full screen".to_string(),
        });
    }
    let windows = window_info_method::list()?;
    let window = window_info_method::find(&windows, window_id, pid, title)?
        .ok_or_else(|| Error::Window("no window matched the given selector".to_string()))?;
    window_capture(window)
}

// needed helper:
fn window_capture(window: WindowInfo) -> Result<Capture, Error> {
    if window.width == 0 || window.height == 0 {
        return Err(Error::Window(format!(
            "window {} has no known geometry to record",
            window.id
        )));
    }
    let size = format!("{}x{}", window.width, window.height);
    let offset = format!("+{},{}", window.x, window.y);
    Ok(Capture {
        size,
        offset,
        desc: format!("window {} ({})", window.id, window.title),
    })
}

// needed helper:
fn screen_size() -> Result<String, Error> {
    let out = Command::new("xdpyinfo")
        .output()
        .map_err(|e| Error::Screenshot(format!("xdpyinfo: {e}")))?;
    let text = String::from_utf8_lossy(&out.stdout);
    for line in text.lines() {
        if let Some(rest) = line.trim().strip_prefix("dimensions:") {
            let size = rest.split_whitespace().next().unwrap_or_default();
            if !size.is_empty() {
                return Ok(size.to_string());
            }
        }
    }
    Err(Error::Screenshot(
        "could not determine screen size via xdpyinfo".to_string(),
    ))
}

// needed helper:
fn max_secs() -> u64 {
    max_secs_from(std::env::var("RECORDING_MAX_SECS").ok())
}

// needed helper:
fn max_secs_from(env_value: Option<String>) -> u64 {
    env_value
        .and_then(|v| v.trim().parse().ok())
        .filter(|&s| s > 0)
        .unwrap_or(DEFAULT_MAX_SECS)
}

// needed helper:
fn video_path(artifacts_dir: Option<&str>) -> String {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    match artifacts_dir {
        Some(dir) => format!("{dir}/{ts}.mp4"),
        None => format!("/tmp/deskctrl-mcp-recording-{ts}.mp4"),
    }
}

// needed helper:
fn spawn_ffmpeg(capture: &Capture, path: &str, max_secs: u64) -> Result<Child, Error> {
    let display = std::env::var("DISPLAY").unwrap_or_else(|_| ":0".to_string());
    let input = format!("{display}{}", capture.offset);
    let cap = max_secs.to_string();
    Command::new("ffmpeg")
        .args([
            "-y",
            "-f",
            "x11grab",
            "-framerate",
            FRAME_RATE,
            "-video_size",
            &capture.size,
            "-i",
            &input,
            "-c:v",
            "libx264",
            "-preset",
            "ultrafast",
            "-crf",
            CRF,
            "-pix_fmt",
            "yuv420p",
            "-t",
            &cap,
            path,
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| Error::Ffmpeg(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::{DEFAULT_MAX_SECS, max_secs_from, video_path};

    #[test]
    fn test_usage() {
        assert_eq!(max_secs_from(None), DEFAULT_MAX_SECS);
        assert_eq!(max_secs_from(Some("300".to_string())), 300);
        assert_eq!(max_secs_from(Some("  60  ".to_string())), 60);
        assert_eq!(max_secs_from(Some("0".to_string())), DEFAULT_MAX_SECS);
        assert_eq!(max_secs_from(Some("abc".to_string())), DEFAULT_MAX_SECS);

        let path = video_path(None);
        assert!(path.ends_with(".mp4") && path.contains("deskctrl-mcp-recording-"));
        let dir = video_path(Some("/tmp/rec"));
        assert!(dir.ends_with(".mp4") && dir.starts_with("/tmp/rec/"));
    }
}
