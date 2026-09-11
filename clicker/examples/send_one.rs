use std::path::PathBuf;

use clicker_lib::testing;

fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("send_one: {err}");
            std::process::ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let path = std::env::args()
        .nth(1)
        .ok_or_else(|| "usage: send_one <path.png|path.mp4>".to_string())?;
    let path = PathBuf::from(path);
    let Some(creds) = testing::load_creds() else {
        return Err("missing TELEGRAM_BOT_TOKEN/TELEGRAM_CHAT_ID in clicker/.env".to_string());
    };
    let caption = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    let id = send_any(&creds, &path, &caption)?;
    println!("sent {} as message {id}", path.display());
    Ok(())
}

// needed helper: routes a single preview to the matching Telegram sender
fn send_any(
    creds: &testing::Creds,
    path: &std::path::Path,
    caption: &str,
) -> Result<String, String> {
    if path.extension().is_some_and(|ext| ext == "mp4") {
        testing::send_video_file(creds, path, caption)
    } else {
        testing::send_photo_file(creds, path, caption)
    }
}
