use serde_json::{Value, json};

use crate::Error;

pub fn call_brp(port: u16, method: &str, params: Option<Value>) -> Result<Value, Error> {
    let url = format!("http://127.0.0.1:{port}");
    let mut request = json!({ "jsonrpc": "2.0", "id": 1, "method": method });
    if let (Some(params), Some(fields)) = (params, request.as_object_mut()) {
        fields.insert("params".to_string(), params);
    }
    let body = ureq::post(&url)
        .header("content-type", "application/json")
        .send(request.to_string())
        .map_err(|e| Error::Http(format!("{method}: {e}")))?
        .body_mut()
        .with_config()
        .limit(64 * 1024 * 1024)
        .read_to_string()
        .map_err(|e| Error::Http(format!("{method}: {e}")))?;
    let trimmed = body.trim();
    let text = trimmed.strip_prefix("data:").unwrap_or(trimmed);
    let reply: Value = serde_json::from_str(text.trim())?;
    if let Some(error) = reply.get("error") {
        return Err(Error::Brp(format!("{method}: {error}")));
    }
    Ok(reply.get("result").cloned().unwrap_or(Value::Null))
}

// no test_usage necessary
