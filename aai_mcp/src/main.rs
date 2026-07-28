#[path = "Error.rs"]
pub mod error;
#[path = "PidParam.rs"]
pub mod pid_param;
#[path = "ProcessHandle.rs"]
pub mod process_handle;
#[path = "ProcessMap.rs"]
pub mod process_map;
#[path = "ReadOutputParams.rs"]
pub mod read_output_params;
#[path = "Server.rs"]
pub mod server;
#[path = "SpawnParams.rs"]
pub mod spawn_params;
#[path = "WriteStdinParams.rs"]
pub mod write_stdin_params;

pub mod ErrorMethod;
pub mod ServerMethod;

pub use error::Error;
pub use pid_param::PidParam;
pub use process_handle::ProcessHandle;
pub use process_map::ProcessMap;
pub use read_output_params::ReadOutputParams;
pub use server::Server;
pub use spawn_params::SpawnParams;
pub use write_stdin_params::WriteStdinParams;

use rmcp::{ServiceExt, transport::stdio};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    let running = Server::new().serve(stdio()).await?;
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
