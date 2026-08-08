use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ReadOutputParams {
    /// Managed process ID
    pub pid: u32,
    /// How long to wait for new output (milliseconds)
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
    /// Return at most this many bytes, keeping the most recent output. A chatty process can
    /// produce megabytes between calls; raise this only when you truly need the whole backlog.
    #[serde(default = "default_max_bytes")]
    pub max_bytes: usize,
}

fn default_timeout() -> u64 {
    200
}

fn default_max_bytes() -> usize {
    32 * 1024
}

#[cfg(test)]
mod tests {
    use crate::ReadOutputParams;

    #[test]
    fn test_usage() {
        let params: ReadOutputParams = serde_json::from_str(r#"{"pid":1}"#).unwrap();
        assert_eq!(params.timeout_ms, 200);
        assert_eq!(params.max_bytes, 32 * 1024);
    }
}
