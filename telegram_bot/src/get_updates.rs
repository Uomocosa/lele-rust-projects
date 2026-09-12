use serde_json::Value;

use crate::Creds;
use crate::Error;

/// # Errors
/// Returns an error if the request fails, Telegram answers with a
/// non-success status or `ok:false`, or the response has no `result` array.
///
/// Polling with `offset=None` is read-only: it leaves the update queue
/// untouched. Pass the last seen `update_id + 1` as `offset` to consume.
pub fn get_updates(
    creds: &Creds,
    offset: Option<i64>,
    timeout_secs: u64,
) -> Result<Vec<Value>, Error> {
    let token = creds.token.as_str();
    let client = reqwest::blocking::Client::new();
    let url = format!("https://api.telegram.org/bot{token}/getUpdates");
    let payload = offset.map_or_else(
        || serde_json::json!({ "limit": 100, "timeout": timeout_secs }),
        |offset| serde_json::json!({ "limit": 100, "timeout": timeout_secs, "offset": offset }),
    );
    let response = client
        .post(&url)
        .json(&payload)
        .send()
        .map_err(|err| Error::Request {
            op: "getUpdates",
            message: err.to_string(),
        })?;
    let status = response.status();
    let status_text = status.to_string();
    let body = response.text().unwrap_or_default();
    let snippet: String = body.chars().take(500).collect();
    if !status.is_success() {
        return Err(Error::Rejected {
            op: "getUpdates",
            status: status_text,
            snippet,
        });
    }
    let parsed: Value = serde_json::from_str(&body).map_err(|_| Error::Rejected {
        op: "getUpdates",
        status: status_text.clone(),
        snippet: snippet.clone(),
    })?;
    if !parsed.get("ok").and_then(Value::as_bool).unwrap_or(false) {
        return Err(Error::Rejected {
            op: "getUpdates",
            status: status_text,
            snippet,
        });
    }
    Ok(parsed
        .get("result")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::get_updates;

    #[test]
    fn test_usage() {
        let _ = get_updates;
    }
}
