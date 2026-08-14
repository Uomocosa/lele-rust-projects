use rmcp::model::{CallToolResult, ContentBlock};
use std::path::Path;

use crate::Error;
use crate::RunPipelineParams;
use crate::server_method;

pub async fn run_pipeline(
    repo: &str,
    token: Option<&str>,
    params: RunPipelineParams,
) -> Result<CallToolResult, Error> {
    let crate_folder = params
        .crate_folder
        .unwrap_or_else(|| "freenet_libp2p_bevy_example_1".to_string());
    let crate_root = format!(
        "{}/../{crate_folder}/Cargo.toml",
        env!("CARGO_MANIFEST_DIR")
    );
    if !Path::new(&crate_root).exists() {
        return Err(Error::CrateNotFound(crate_folder, crate_root));
    }
    let run_tests = params.run_tests.unwrap_or(true);
    let release_builds = params.release_builds.unwrap_or(true);
    let build_mode = params.build_mode.unwrap_or_else(|| "dev".to_string());
    let args = [
        "workflow".to_string(),
        "run".to_string(),
        "self-hosted-ci.yml".to_string(),
        "-R".to_string(),
        repo.to_string(),
        "-f".to_string(),
        format!("crate={crate_folder}"),
        "-f".to_string(),
        format!("run-tests={run_tests}"),
        "-f".to_string(),
        format!("release-builds={release_builds}"),
        "-f".to_string(),
        format!("build-mode={build_mode}"),
    ];
    server_method::run_gh(token, &args).await?;
    Ok(CallToolResult::success(vec![ContentBlock::text(format!(
        "triggered self-hosted pipeline for {crate_folder} (run-tests={run_tests}, release-builds={release_builds}, build-mode={build_mode})"
    ))]))
}

#[cfg(test)]
mod tests {
    use super::run_pipeline;
    use crate::RunPipelineParams;

    #[tokio::test]
    async fn test_usage() {
        let params = RunPipelineParams {
            crate_folder: Some("no_such_crate".into()),
            run_tests: None,
            release_builds: None,
            build_mode: None,
        };
        let err = run_pipeline("repo", None, params).await.unwrap_err();
        assert!(err.to_string().contains("no_such_crate"));
    }
}
