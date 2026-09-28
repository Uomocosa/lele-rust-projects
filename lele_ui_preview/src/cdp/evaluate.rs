use serde_json::{Value, json};

use crate::Error;
use crate::cdp;

pub fn evaluate(session: &mut cdp::Session, expression: &str) -> Result<Value, Error> {
    let result = cdp::call(
        session,
        "Runtime.evaluate",
        &json!({ "expression": expression, "returnByValue": true, "awaitPromise": true }),
    )?;
    if let Some(details) = result.get("exceptionDetails") {
        return Err(Error::Cdp(format!("evaluate failed: {details}")));
    }
    Ok(result
        .get("result")
        .and_then(|inner| inner.get("value"))
        .cloned()
        .unwrap_or(Value::Null))
}

// no test_usage necessary
