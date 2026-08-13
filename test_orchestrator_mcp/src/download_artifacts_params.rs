use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DownloadArtifactsParams {
    /// Workflow run id; defaults to the latest run
    #[serde(default)]
    pub run_id: Option<u64>,
    /// Artifact name pattern to download; defaults to all artifacts of the run
    pub pattern: Option<String>,
    /// Destination directory; defaults to "downloads" under the current dir
    pub dest: Option<String>,
}
