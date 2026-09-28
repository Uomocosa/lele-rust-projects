use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fixture {
    pub label: String,
    pub value: Value,
}
