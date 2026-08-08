#[path = "ClickParams.rs"]
pub mod click_params;
#[path = "Error.rs"]
pub mod error;
#[path = "OutputBuffer.rs"]
pub mod output_buffer;
#[path = "PidParam.rs"]
pub mod pid_param;
#[path = "ProcessHandle.rs"]
pub mod process_handle;
#[path = "ProcessMap.rs"]
pub mod process_map;
#[path = "ReadOutputParams.rs"]
pub mod read_output_params;
#[path = "ScreenshotParams.rs"]
pub mod screenshot_params;
#[path = "SendToTelegramParams.rs"]
pub mod send_to_telegram_params;
#[path = "Server.rs"]
pub mod server;
#[path = "SpawnParams.rs"]
pub mod spawn_params;
#[cfg(test)]
#[path = "TestSupport.rs"]
pub mod test_support;
#[path = "WaitForOutputParams.rs"]
pub mod wait_for_output_params;
#[path = "WindowInfo.rs"]
pub mod window_info;
#[path = "WriteStdinParams.rs"]
pub mod write_stdin_params;

pub mod ErrorMethod;
pub mod OutputBufferMethod;
pub mod ServerMethod;
pub mod WindowInfoMethod;

use crate::send_to_telegram_params::SendToTelegramParams;
pub use click_params::ClickParams;
pub use error::Error;
pub use output_buffer::OutputBuffer;
pub use pid_param::PidParam;
pub use process_handle::ProcessHandle;
pub use process_map::ProcessMap;
pub use read_output_params::ReadOutputParams;
pub use screenshot_params::ScreenshotParams;
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
    let running = Server::with_artifacts_dir(artifacts_dir)
        .serve(stdio())
        .await?;
    running.waiting().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {
        // trivial binary wrapper; real coverage is the stdio JSON-RPC smoke test
    }
}
