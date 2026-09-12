use serde_json::Value;

#[must_use]
pub fn latest_message<'u>(updates: &'u [Value], chat_id: &str) -> Option<&'u Value> {
    let wanted: i64 = chat_id.parse().ok()?;
    updates.iter().rev().find_map(|update| {
        let message = ["message", "edited_message", "channel_post"]
            .into_iter()
            .find_map(|key| update.get(key))?;
        let id = message
            .get("chat")
            .and_then(|chat| chat.get("id"))
            .and_then(Value::as_i64)?;
        (id == wanted).then_some(message)
    })
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::latest_message;

    #[test]
    fn test_usage() {
        let updates = serde_json::json!([
            {"update_id": 1, "message": {"message_id": 1, "chat": {"id": 111}, "text": "old"}},
            {"update_id": 2, "message": {"message_id": 2, "chat": {"id": 222}, "text": "other"}},
            {"update_id": 3, "message": {"message_id": 3, "chat": {"id": 111}, "text": "new"}},
        ]);
        let updates: &Vec<Value> = updates.as_array().unwrap();
        let latest = latest_message(updates, "111").unwrap();
        assert_eq!(latest.get("text").and_then(Value::as_str), Some("new"));
        assert!(latest_message(updates, "999").is_none());
        assert!(latest_message(updates, "not-a-number").is_none());
        assert!(latest_message(&[], "111").is_none());
    }
}
