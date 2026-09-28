use serde_json::{Value, json};
use tungstenite::Message;

use crate::Error;
use crate::cdp;

pub fn call(session: &mut cdp::Session, method: &str, params: &Value) -> Result<Value, Error> {
    session.next_id = session.next_id.wrapping_add(1);
    let id = session.next_id;
    let request = json!({ "id": id, "method": method, "params": params });
    session
        .socket
        .send(Message::text(request.to_string()))
        .map_err(|e| Error::Cdp(format!("{method}: send: {e}")))?;
    loop {
        let message = session
            .socket
            .read()
            .map_err(|e| Error::Cdp(format!("{method}: read: {e}")))?;
        let Message::Text(text) = message else {
            continue;
        };
        let reply: Value = serde_json::from_str(text.as_str())?;
        if reply.get("id").and_then(Value::as_u64) != Some(id) {
            continue;
        }
        if let Some(error) = reply.get("error") {
            return Err(Error::Cdp(format!("{method}: {error}")));
        }
        return Ok(reply.get("result").cloned().unwrap_or(Value::Null));
    }
}

// no test_usage necessary
