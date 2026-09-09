use std::path::PathBuf;

use clicker_lib::testing;

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
    let Some(creds) = testing::load_creds() else {
        return Err("missing TELEGRAM_BOT_TOKEN/TELEGRAM_CHAT_ID in clicker/.env".to_string());
    };
    let files = previews()?;
    if files.is_empty() {
        return Err("no *.png previews at crate root; run ui_png tests first".to_string());
    }
    for path in &files {
        let caption = path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();
        let id = testing::send_photo_file(&creds, path, &caption)?;
        println!("sent {} as message {id}", path.display());
    }
    Ok(())
}

fn previews() -> Result<Vec<PathBuf>, String> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let entries =
        std::fs::read_dir(&dir).map_err(|err| format!("read {}: {err}", dir.display()))?;
    let mut files = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|err| format!("read entry in {}: {err}", dir.display()))?;
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "png") {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}
