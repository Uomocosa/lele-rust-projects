use serde_json::Value;

/// # Errors
/// Returns an error when the BRP request cannot be sent, decoded, or carries a
/// protocol error.
pub fn request(port: u16, method: &str, params: &Value) -> Result<Value, String> {
    let url = format!("http://127.0.0.1:{port}/");
    let envelope = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1u8,
        "method": method,
        "params": params,
    });
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|err| format!("brp client: {err}"))?;
    let reply: Value = client
        .post(url)
        .json(&envelope)
        .send()
        .map_err(|err| format!("brp send: {err}"))?
        .json()
        .map_err(|err| format!("brp decode: {err}"))?;
    if let Some(error) = reply.get("error") {
        return Err(format!("brp error: {error}"));
    }
    reply
        .get("result")
        .cloned()
        .ok_or_else(|| "brp response missing result".to_string())
}

// no test_usage necessary
