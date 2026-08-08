use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct WaitForOutputParams {
    /// The process ID returned by spawn_process.
    pub pid: u32,
    /// Plain substring to wait for (not a regex). Matched against whole lines.
    pub pattern: String,
    /// How long to wait, in milliseconds. Capped at 120000; call again to keep waiting.
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
}

fn default_timeout_ms() -> u64 {
    30_000
}

#[cfg(test)]
mod tests {
    use crate::WaitForOutputParams;

    #[test]
    fn test_usage() {
        let params: WaitForOutputParams =
            serde_json::from_str(r#"{"pid":1,"pattern":"connected, running"}"#).unwrap();
        assert_eq!(params.timeout_ms, 30_000);
        assert_eq!(params.pattern, "connected, running");
    }
}
