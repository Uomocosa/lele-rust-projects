use std::{fs, process::Command, time::SystemTime};

use base64::{Engine, engine::general_purpose::STANDARD};
use rmcp::model::{CallToolResult, ContentBlock};

use crate::{Error, ScreenshotParams, WindowInfo, window_info_method};

use super::telegram;

pub async fn screenshot(
    params: ScreenshotParams,
    artifacts_dir: Option<&str>,
    bot_token: Option<&str>,
    chat_id: Option<&str>,
    send_to_telegram: bool,
) -> Result<CallToolResult, Error> {
    let target = resolve_target(&params)?;

    // Full-screen capture may find a blanked/locked screen; a window we were handed by id is
    // already known to exist, so skip the wake dance and its 5s settle.
    if target.is_none() {
        wake_screen();
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }

    let png = match &target {
        Some(window) => capture_window_png(window)?,
        None => capture_png()?,
    };
    let (width, height) = png_size(&png).unwrap_or((0, 0));
    let summary = match &target {
        Some(w) => format!(
            "captured window {} (pid {}, {}) PNG ({width}x{height}, {} KB)",
            w.id,
            w.pid,
            w.title,
            png.len() / 1024
        ),
        None => format!(
            "captured primary monitor PNG ({width}x{height}, {} KB)",
            png.len() / 1024
        ),
    };

    if let Some(dir) = artifacts_dir {
        let _ = fs::create_dir_all(dir);
        let ts = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let path = format!("{dir}/{ts}.png");
        let _ = fs::write(&path, &png);
    }

    if let (Some(token), Some(cid)) = (bot_token, chat_id)
        && send_to_telegram
    {
        // One rich photo message: the agent's caption (template) or an auto summary.
        let caption = params
            .caption
            .clone()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| summary.clone());
        telegram::send_photo_caption_fire_and_forget(
            token.to_string(),
            cid.to_string(),
            png.clone(),
            Some(caption),
        );
    }

    Ok(CallToolResult::success(vec![
        ContentBlock::text(summary),
        ContentBlock::image(STANDARD.encode(&png), "image/png"),
    ]))
}

/// `Ok(None)` means "no selector given" — capture the whole screen.
// needed helper:
fn resolve_target(params: &ScreenshotParams) -> Result<Option<WindowInfo>, Error> {
    if params.window_id.is_none() && params.pid.is_none() && params.title.is_none() {
        return Ok(None);
    }
    if let Some(id) = &params.window_id
        && !window_info_method::is_valid_id(id)
    {
        return Err(Error::Window(format!(
            "invalid window_id {id:?}: expected hex like \"0x03a00004\" (see list_windows)"
        )));
    }
    // Only pay for the wmctrl call when a selector was actually given.
    let windows = window_info_method::list()?;
    window_info_method::find(
        &windows,
        params.window_id.as_deref(),
        params.pid,
        params.title.as_deref(),
    )
}

// needed helper:
fn wake_screen() {
    let display = std::env::var("DISPLAY").unwrap_or_else(|_| ":0".to_string());
    let _ = Command::new("cinnamon-screensaver-command")
        .arg("--deactivate")
        .output();
    let _ = Command::new("gnome-screensaver-command")
        .arg("--deactivate")
        .output();
    let _ = Command::new("xdg-screensaver").arg("reset").output();
    let _ = Command::new("xset")
        .args(["-display", &display, "dpms", "force", "on"])
        .output();
    let _ = Command::new("loginctl").arg("unlock-session").output();
}

// needed helper:
fn png_size(bytes: &[u8]) -> Option<(u32, u32)> {
    if bytes.len() < 24 || &bytes[12..16] != b"IHDR" {
        return None;
    }
    let width = u32::from_be_bytes(bytes[16..20].try_into().ok()?);
    let height = u32::from_be_bytes(bytes[20..24].try_into().ok()?);
    Some((width, height))
}

// needed helper:
fn temp_path() -> String {
    format!("/tmp/deskctrl-mcp-shot-{}.png", std::process::id())
}

// needed helper:
fn capture_png() -> Result<Vec<u8>, Error> {
    let path = temp_path();

    let ok = try_gnome_screenshot(&path)
        .or_else(|_| try_import(&path))
        .or_else(|_| try_xwd(&path))
        .map_err(Error::Screenshot)?;

    if !ok {
        return Err(Error::Screenshot(
            "all screenshot commands reported failure".to_string(),
        ));
    }

    read_and_clean(&path)
}

/// Under a compositor `import -window` returns the window's own redirected pixels even when it
/// is occluded, so it is tried first. Raising the window steals focus, so that is last.
// needed helper:
fn capture_window_png(window: &WindowInfo) -> Result<Vec<u8>, Error> {
    let path = temp_path();

    let ok = try_import_window(&window.id, &path)
        .or_else(|_| try_xwd_window(&window.id, &path))
        .or_else(|_| try_raise_and_crop(window, &path))
        .map_err(Error::Screenshot)?;

    if !ok {
        return Err(Error::Screenshot(format!(
            "could not capture window {} ({})",
            window.id, window.title
        )));
    }

    read_and_clean(&path)
}

// needed helper:
fn read_and_clean(path: &str) -> Result<Vec<u8>, Error> {
    let bytes = fs::read(path).map_err(|e| Error::Screenshot(format!("read screenshot: {e}")))?;
    let _ = fs::remove_file(path);
    Ok(bytes)
}

// needed helper:
fn try_gnome_screenshot(path: &str) -> Result<bool, String> {
    let out = Command::new("gnome-screenshot")
        .args(["-f", path])
        .output()
        .map_err(|e| e.to_string())?;
    Ok(out.status.success())
}

// needed helper:
fn try_import(path: &str) -> Result<bool, String> {
    let out = Command::new("import")
        .args(["-window", "root", path])
        .output()
        .map_err(|e| e.to_string())?;
    Ok(out.status.success())
}

// needed helper:
fn try_import_window(id: &str, path: &str) -> Result<bool, String> {
    let out = Command::new("import")
        .args(["-window", id, path])
        .output()
        .map_err(|e| e.to_string())?;
    Ok(out.status.success())
}

// needed helper:
fn try_xwd(path: &str) -> Result<bool, String> {
    xwd_to_png(&["-root", "-silent"], path)
}

// needed helper:
fn try_xwd_window(id: &str, path: &str) -> Result<bool, String> {
    xwd_to_png(&["-id", id, "-silent"], path)
}

// needed helper:
fn xwd_to_png(args: &[&str], path: &str) -> Result<bool, String> {
    let xwd_path = format!("{path}.xwd");
    let mut cmd = Command::new("xwd");
    cmd.args(args).args(["-out", &xwd_path]);
    let xwd_ok = cmd.output().map_err(|e| e.to_string())?.status.success();
    if !xwd_ok {
        return Ok(false);
    }
    let ok = Command::new("convert")
        .args([&xwd_path, path])
        .output()
        .map_err(|e| e.to_string())?
        .status
        .success();
    let _ = fs::remove_file(&xwd_path);
    Ok(ok)
}

/// Last resort: raise the window (steals focus), grab the root, crop to its geometry.
// needed helper:
fn try_raise_and_crop(window: &WindowInfo, path: &str) -> Result<bool, String> {
    if window.width == 0 || window.height == 0 {
        return Err(format!(
            "window {} has no known geometry to crop to",
            window.id
        ));
    }

    let raised = Command::new("wmctrl")
        .args(["-i", "-a", &window.id])
        .output()
        .map_err(|e| e.to_string())?
        .status
        .success();
    if !raised {
        return Ok(false);
    }
    std::thread::sleep(std::time::Duration::from_millis(400));

    let full = format!("{path}.full.png");
    if !try_import(&full)? {
        return Ok(false);
    }
    let ok = Command::new("convert")
        .args([&full, "-crop", &window.geometry(), "+repage", path])
        .output()
        .map_err(|e| e.to_string())?
        .status
        .success();
    let _ = fs::remove_file(&full);
    Ok(ok)
}

#[cfg(test)]
mod tests {
    use super::png_size;
    use crate::ScreenshotParams;

    #[test]
    fn test_usage() {
        let mut bytes = vec![0u8; 24];
        bytes[12..16].copy_from_slice(b"IHDR");
        bytes[16..20].copy_from_slice(&1920u32.to_be_bytes());
        bytes[20..24].copy_from_slice(&1080u32.to_be_bytes());
        assert_eq!(png_size(&bytes), Some((1920, 1080)));
        assert_eq!(png_size(&[0u8; 8]), None);

        // No selectors means full screen, without touching wmctrl.
        assert!(
            super::resolve_target(&ScreenshotParams::default())
                .unwrap()
                .is_none()
        );
        // A malformed id is rejected before any command runs.
        let bad = ScreenshotParams {
            window_id: Some("not-an-id".to_string()),
            ..Default::default()
        };
        assert!(super::resolve_target(&bad).is_err());
    }

    #[tokio::test]
    async fn test_usage_live_display() {
        crate::test_support::assert_live_display();
        let _guard = crate::test_support::live_test_lock().lock().await;

        let result = super::screenshot(ScreenshotParams::default(), None, None, None, true).await;
        assert!(result.is_ok());
    }

    /// Spawns a real xterm and screenshots it by pid, asserting the returned PNG has a nonzero
    /// size.
    #[tokio::test]
    async fn test_usage_live_window_capture() {
        crate::test_support::assert_live_display();
        let _guard = crate::test_support::live_test_lock().lock().await;

        let mut child = std::process::Command::new("xterm")
            .spawn()
            .expect("spawning xterm for live screenshot test");
        std::thread::sleep(std::time::Duration::from_millis(800));

        let params = ScreenshotParams {
            pid: Some(child.id()),
            ..Default::default()
        };
        let result = super::screenshot(params, None, None, None, true).await;

        let _ = child.kill();
        let _ = child.wait();

        let result = result.expect("live window screenshot");
        let png_content = result
            .content
            .iter()
            .find_map(|c| c.as_image())
            .expect("screenshot result missing image content");
        use base64::Engine;
        let png = base64::engine::general_purpose::STANDARD
            .decode(&png_content.data)
            .expect("decoding screenshot base64");
        let (w, h) = png_size(&png).expect("parsing PNG header");
        assert!(
            w > 0 && h > 0,
            "expected nonzero window screenshot, got {w}x{h}"
        );
    }
}
