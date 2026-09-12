use std::path::PathBuf;

use telegram_bot;

fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("send_previews: {err}");
            std::process::ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let Some(creds) = telegram_bot::load_creds() else {
        return Err("missing TELEGRAM_BOT_TOKEN/TELEGRAM_CHAT_ID in clicker/.env".to_string());
    };
    let files = previews()?;
    if files.is_empty() {
        return Err("no previews at crate root; run ui_png/ui_mp4 tests first".to_string());
    }
    for path in &files {
        let caption = path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();
        let id = send_any(&creds, path, &caption)?;
        println!("sent {} as message {id}", path.display());
    }
    Ok(())
}

// needed helper: routes previews to the matching Telegram sender
fn send_any(
    creds: &telegram_bot::Creds,
    path: &std::path::Path,
    caption: &str,
) -> Result<String, String> {
    if path.extension().is_some_and(|ext| ext == "mp4") {
        telegram_bot::send_video_file(creds, path, caption).map_err(|err| err.to_string())
    } else {
        telegram_bot::send_photo_file(creds, path, caption).map_err(|err| err.to_string())
    }
}

fn previews() -> Result<Vec<PathBuf>, String> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let entries =
        std::fs::read_dir(&dir).map_err(|err| format!("read {}: {err}", dir.display()))?;
    let mut files = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|err| format!("read entry in {}: {err}", dir.display()))?;
        let path = entry.path();
        if path
            .extension()
            .is_some_and(|ext| ext == "png" || ext == "mp4")
        {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}
