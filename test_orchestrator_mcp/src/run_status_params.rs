use derive_more::Deref;
use serde::{Deserialize, Deserializer};

#[derive(Debug, Clone, Copy, Deref)]
pub struct RunStatusParams(pub Option<u64>);

impl<'de> Deserialize<'de> for RunStatusParams {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Inner {
            /// Workflow run id; defaults to the latest run
            #[serde(default)]
            run_id: Option<u64>,
        }
        Inner::deserialize(deserializer).map(|inner| Self(inner.run_id))
    }
}

#[rustfmt::skip]
impl schemars::JsonSchema for RunStatusParams {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "RunStatusParams".into()
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        let mut properties = serde_json::Map::new();
        properties.insert(
            "run_id".to_string(),
            generator.subschema_for::<Option<u64>>().as_value().clone(),
        );
        let mut object = serde_json::Map::new();
        object.insert("type".to_string(), "object".into());
        object.insert("properties".to_string(), properties.into());
        object.into()
    }
}
