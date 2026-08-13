use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct TriggerTagCiParams {
    /// One of test, build, release, release-notests
    pub mode: String,
    /// Crate folder name; defaults to freenet_libp2p_bevy_example_1
    #[serde(default, rename = "crate")]
    pub crate_folder: Option<String>,
    /// Only compute and report the tag, do not push it
    #[serde(default)]
    pub dry_run: Option<bool>,
}
