use serde_json::Value;

use crate::Error;

pub fn message_ids(op: &'static str, status: &str, body: &str) -> Result<Vec<String>, Error> {
    let snippet: String = body.chars().take(500).collect();
    let rejected = || Error::Rejected {
        op,
        status: status.to_string(),
        snippet: snippet.clone(),
    };
    let parsed: Value = serde_json::from_str(body).map_err(|_| rejected())?;
    if !parsed.get("ok").and_then(Value::as_bool).unwrap_or(false) {
        return Err(rejected());
    }
    let messages = parsed
        .get("result")
        .and_then(Value::as_array)
        .ok_or_else(rejected)?;
    Ok(messages
        .iter()
        .map(|message| {
            message
                .get("message_id")
                .and_then(Value::as_u64)
                .map_or_else(|| "ok".to_string(), |id| id.to_string())
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::message_ids;

    #[test]
    fn test_usage() {
        let body = r#"{"ok":true,"result":[{"message_id":7},{"message_id":8}]}"#;
        assert_eq!(message_ids("op", "200", body).unwrap(), vec!["7", "8"]);
        assert!(message_ids("op", "200", r#"{"ok":true,"result":{"message_id":1}}"#).is_err());
        assert!(message_ids("op", "200", r#"{"ok":false}"#).is_err());
        assert!(message_ids("op", "200", "garbage").is_err());
    }
}
