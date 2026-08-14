pub mod download_artifacts_params;
pub mod error;
pub mod error_method;
pub mod game_status_params;
pub mod launch_game_params;
pub mod next_tag_params;
pub mod probe_network_params;
pub mod run_pipeline_params;
pub mod run_status_params;
pub mod server;
pub mod server_method;
pub mod stop_game_params;
pub mod trigger_tag_ci_params;

pub use download_artifacts_params::DownloadArtifactsParams;
pub use error::Error;
pub use game_status_params::GameStatusParams;
pub use launch_game_params::LaunchGameParams;
pub use next_tag_params::NextTagParams;
pub use probe_network_params::ProbeNetworkParams;
pub use run_pipeline_params::RunPipelineParams;
pub use run_status_params::RunStatusParams;
pub use server::Server;
pub use stop_game_params::StopGameParams;
pub use trigger_tag_ci_params::TriggerTagCiParams;

use rmcp::{ServiceExt, transport::stdio};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::from_filename(concat!(env!("CARGO_MANIFEST_DIR"), "/.env"))
        .or_else(|_| dotenvy::dotenv());

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    let server = Server::new();
    let running = server.serve(stdio()).await?;
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
