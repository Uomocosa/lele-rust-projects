use serde_json::Value;

use crate::Error;

pub(crate) fn message_id(op: &'static str, status: &str, body: &str) -> Result<String, Error> {
    let snippet: String = body.chars().take(500).collect();
    let parsed: Value = serde_json::from_str(body).map_err(|_| Error::Rejected {
        op,
        status: status.to_string(),
        snippet: snippet.clone(),
    })?;
    if !parsed.get("ok").and_then(Value::as_bool).unwrap_or(false) {
        return Err(Error::Rejected {
            op,
            status: status.to_string(),
            snippet,
        });
    }
    Ok(parsed
        .get("result")
        .and_then(|result| result.get("message_id"))
        .and_then(Value::as_u64)
        .map_or_else(|| "ok".to_string(), |id| id.to_string()))
}

#[cfg(test)]
mod tests {
    use super::message_id;

    #[test]
    fn test_usage() {
        assert_eq!(
            message_id("op", "200", r#"{"ok":true,"result":{"message_id":42}}"#).unwrap(),
            "42"
        );
        assert_eq!(message_id("op", "200", r#"{"ok":true}"#).unwrap(), "ok");
        assert!(message_id("op", "200", "garbage").is_err());
        assert!(message_id("op", "200", r#"{"ok":false,"description":"bad"}"#).is_err());
    }
}
