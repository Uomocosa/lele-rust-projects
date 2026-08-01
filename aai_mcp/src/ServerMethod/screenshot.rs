use std::{fs, process::Command, time::SystemTime};

use base64::{Engine, engine::general_purpose::STANDARD};
use rmcp::model::{CallToolResult, ContentBlock};

use crate::Error;

use super::send_to_telegram_send;

pub async fn screenshot(
    artifacts_dir: Option<&str>,
    bot_token: Option<&str>,
    chat_id: Option<&str>,
) -> Result<CallToolResult, Error> {
    wake_screen();
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    let png = capture_png()?;
    let (width, height) = png_size(&png).unwrap_or((0, 0));
    let summary = format!(
        "captured primary monitor PNG ({width}x{height}, {} KB)",
        png.len() / 1024
    );

    if let Some(dir) = artifacts_dir {
        let _ = fs::create_dir_all(dir);
        let ts = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let path = format!("{dir}/{ts}.png");
        let _ = fs::write(&path, &png);
    }

    if let (Some(token), Some(cid)) = (bot_token, chat_id) {
        send_to_telegram_send::send_photo_fire_and_forget(
            token.to_string(),
            cid.to_string(),
            png.clone(),
        );
    }

    Ok(CallToolResult::success(vec![
        ContentBlock::text(summary),
        ContentBlock::image(STANDARD.encode(&png), "image/png"),
    ]))
}

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

fn png_size(bytes: &[u8]) -> Option<(u32, u32)> {
    if bytes.len() < 24 || &bytes[12..16] != b"IHDR" {
        return None;
    }
    let width = u32::from_be_bytes(bytes[16..20].try_into().ok()?);
    let height = u32::from_be_bytes(bytes[20..24].try_into().ok()?);
    Some((width, height))
}

fn capture_png() -> Result<Vec<u8>, Error> {
    let path = format!("/tmp/aai-mcp-shot-{}.png", std::process::id());

    let ok = try_gnome_screenshot(&path)
        .or_else(|_| try_import(&path))
        .or_else(|_| try_xwd(&path))
        .map_err(Error::Screenshot)?;

    if !ok {
        return Err(Error::Screenshot(
            "all screenshot commands reported failure".to_string(),
        ));
    }

    let bytes = fs::read(&path).map_err(|e| Error::Screenshot(format!("read screenshot: {e}")))?;
    let _ = fs::remove_file(&path);
    Ok(bytes)
}

fn try_gnome_screenshot(path: &str) -> Result<bool, String> {
    let out = Command::new("gnome-screenshot")
        .args(["-f", path])
        .output()
        .map_err(|e| e.to_string())?;
    Ok(out.status.success())
}

fn try_import(path: &str) -> Result<bool, String> {
    let out = Command::new("import")
        .args(["-window", "root", path])
        .output()
        .map_err(|e| e.to_string())?;
    Ok(out.status.success())
}

fn try_xwd(path: &str) -> Result<bool, String> {
    let xwd_path = format!("{path}.xwd");
    let xwd_ok = Command::new("xwd")
        .args(["-root", "-silent", "-out", &xwd_path])
        .output()
        .map_err(|e| e.to_string())?
        .status
        .success();
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

#[cfg(test)]
mod tests {
    use super::png_size;

    #[test]
    fn test_usage() {
        let mut bytes = vec![0u8; 24];
        bytes[12..16].copy_from_slice(b"IHDR");
        bytes[16..20].copy_from_slice(&1920u32.to_be_bytes());
        bytes[20..24].copy_from_slice(&1080u32.to_be_bytes());
        assert_eq!(png_size(&bytes), Some((1920, 1080)));
        assert_eq!(png_size(&[0u8; 8]), None);
    }

    #[tokio::test]
    #[ignore = "requires a live X display"]
    async fn test_usage_live_display() {
        let result = super::screenshot(None, None, None).await;
        assert!(result.is_ok());
    }
}
