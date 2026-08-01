use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
pub struct SendToTelegramParams {
    #[schemars(description = "Optional text message to send")]
    pub text: Option<String>,
    #[schemars(description = "Optional base64-encoded PNG to send as a photo")]
    pub photo_base64: Option<String>,
}
