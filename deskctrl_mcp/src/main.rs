pub mod click_params;
pub mod error;
pub mod error_method;
pub mod keyboard_input;
pub mod keyboard_key;
pub mod output_buffer;
pub mod output_buffer_method;
pub mod process_handle;
pub mod process_map;
pub mod read_output_params;
pub mod record_video_params;
pub mod recording;
pub mod recording_method;
pub mod screenshot_params;
pub mod send_keys_params;
pub mod send_to_telegram_params;
pub mod server;
pub mod server_method;
pub mod spawn_params;
#[cfg(test)]
pub mod test_support;
pub mod wait_for_output_params;
pub mod window_info;
pub mod window_info_method;
pub mod write_stdin_params;

mod server_click_window;
mod server_kill_process;
mod server_send_keys;
mod server_spawn_process;
mod server_write_stdin;

pub use click_params::ClickParams;
pub use error::Error;
pub use keyboard_input::KeyboardInput;
pub use keyboard_key::KeyboardKey;
pub use output_buffer::OutputBuffer;
pub use process_handle::ProcessHandle;
pub use process_map::ProcessMap;
pub use read_output_params::ReadOutputParams;
pub use record_video_params::RecordVideoParams;
pub use recording::Recording;
pub use screenshot_params::ScreenshotParams;
pub use send_keys_params::SendKeysParams;
pub use send_to_telegram_params::SendToTelegramParams;
pub use server::Server;
pub use spawn_params::SpawnParams;
pub use wait_for_output_params::WaitForOutputParams;
pub use window_info::WindowInfo;
pub use write_stdin_params::WriteStdinParams;

use rmcp::{ServiceExt, transport::stdio};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::from_filename(concat!(env!("CARGO_MANIFEST_DIR"), "/.env"))
        .or_else(|_| dotenvy::dotenv());

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    let artifacts_dir = std::env::var("AAI_ARTIFACTS_DIR").ok();
    let server = Server::with_artifacts_dir(artifacts_dir);
    let recording = server.recording.clone();
    let bot_token = server.bot_token.clone();
    let chat_id = server.chat_id.clone();
    let artifacts_dir = server.artifacts_dir.clone();

    if let (Some(token), Some(cid)) = (&bot_token, &chat_id) {
        let now = chrono::Local::now().format("%Y_%m_%d [%H:%M:%S]");
        server_method::telegram::send_text_fire_and_forget(
            token.clone(),
            cid.clone(),
            format!("\u{1F4CB} Starting Session \u{2014} {now}"),
        );

        // Auto-record the session so it can be sent as a video at session end. If ffmpeg is
        // missing, start() errors — log it and continue (the record_video tool raises the
        // install error when the agent calls it explicitly).
        match recording_method::start(&recording, artifacts_dir.as_deref(), None, None, None).await
        {
            Ok(desc) => {
                tracing::info!(target: "deskctrl_mcp::recording", "{desc}");
            }
            Err(e) => {
                tracing::warn!(target: "deskctrl_mcp::recording", "auto-record skipped: {e}");
            }
        }
    }

    let running = server.serve(stdio()).await?;
    running.waiting().await?;

    // Session end: the stdio transport has closed, so stop the recording and send the video.
    if let (Some(token), Some(cid)) = (bot_token, chat_id)
        && let Ok(stopped) = recording_method::stop(&recording).await
        && let Ok(mp4) = std::fs::read(&stopped.path)
        && mp4.len() as u64 <= 50 * 1024 * 1024
    {
        let caption = stopped.caption();
        server_method::telegram::send_video_fire_and_forget(token, cid, mp4, caption);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {
        // trivial binary wrapper; real coverage is the stdio JSON-RPC smoke test
    }
}
