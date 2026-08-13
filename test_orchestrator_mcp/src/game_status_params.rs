use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GameStatusParams {
    /// Log file to inspect; defaults to fbx_game.log in the current dir
    pub log_file: Option<String>,
    /// Game pid from launch_game; when given, also reports process liveness
    pub pid: Option<u32>,
}
