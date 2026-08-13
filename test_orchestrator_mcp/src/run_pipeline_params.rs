use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct RunPipelineParams {
    /// Crate folder name, e.g. freenet_libp2p_bevy_example_1
    #[serde(default, rename = "crate")]
    pub crate_folder: Option<String>,
    /// Run the Linux test gate (build, fmt, clippy, tests, subcrates, lele_lint)
    #[serde(default)]
    pub run_tests: Option<bool>,
    /// Build release binaries on Linux + Windows and upload artifacts
    #[serde(default)]
    pub release_builds: Option<bool>,
}
