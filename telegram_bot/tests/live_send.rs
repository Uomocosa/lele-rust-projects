use telegram_bot::load_creds;

fn assert_arrived(id: &str) {
    assert!(
        id.parse::<u64>().is_ok(),
        "expected numeric message_id proving Telegram stored the message, got {id:?}"
    );
}

fn ffmpeg_media(args: &[&str], path: &std::path::Path) -> Option<Vec<u8>> {
    let output = std::process::Command::new("ffmpeg")
        .args(args)
        .arg(path)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let bytes = std::fs::read(path).ok()?;
    let _ = std::fs::remove_file(path);
    Some(bytes)
}

#[test]
#[ignore = "needs TELEGRAM_BOT_TOKEN/TELEGRAM_CHAT_ID; sends a real message"]
fn live_send_text() {
    let Some(creds) = load_creds() else {
        eprintln!("skipping live_send_text: no telegram creds");
        return;
    };
    let id = telegram_bot::send_text(&creds, "telegram_bot live test: send_text OK")
        .expect("live send_text");
    assert_arrived(&id);
}

#[test]
#[ignore = "needs TELEGRAM_BOT_TOKEN/TELEGRAM_CHAT_ID and ffmpeg; sends a real photo"]
fn live_send_photo() {
    let Some(creds) = load_creds() else {
        eprintln!("skipping live_send_photo: no telegram creds");
        return;
    };
    let path = std::env::temp_dir().join(format!("telegram-bot-live-{}.png", std::process::id()));
    let Some(png) = ffmpeg_media(
        &[
            "-y",
            "-f",
            "lavfi",
            "-i",
            "testsrc=duration=1:size=128x128:rate=1",
            "-frames:v",
            "1",
        ],
        &path,
    ) else {
        eprintln!("skipping live_send_photo: ffmpeg unavailable");
        return;
    };
    let caption = "telegram_bot live test: send_photo OK";
    let id = telegram_bot::send_photo(&creds, &png, caption).expect("live send_photo");
    assert_arrived(&id);
}

#[test]
#[ignore = "needs TELEGRAM_BOT_TOKEN/TELEGRAM_CHAT_ID and ffmpeg; sends a real 1s clip"]
fn live_send_video() {
    let Some(creds) = load_creds() else {
        eprintln!("skipping live_send_video: no telegram creds");
        return;
    };
    let path = std::env::temp_dir().join(format!("telegram-bot-live-{}.mp4", std::process::id()));
    let Some(bytes) = ffmpeg_media(
        &[
            "-y",
            "-f",
            "lavfi",
            "-i",
            "testsrc=duration=1:size=64x64:rate=10",
            "-c:v",
            "libx264",
            "-pix_fmt",
            "yuv420p",
        ],
        &path,
    ) else {
        eprintln!("skipping live_send_video: ffmpeg unavailable");
        return;
    };
    let id = telegram_bot::send_video(&creds, &bytes, "telegram_bot live test: send_video OK")
        .expect("live send_video");
    assert_arrived(&id);
}
