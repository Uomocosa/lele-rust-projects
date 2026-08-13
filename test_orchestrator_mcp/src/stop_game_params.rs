use derive_more::Deref;
use serde::{Deserialize, Deserializer};

#[derive(Debug, Clone, Copy, Deref)]
pub struct StopGameParams(pub u32);

impl<'de> Deserialize<'de> for StopGameParams {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Inner {
            /// pid returned by launch_game
            pid: u32,
        }
        Inner::deserialize(deserializer).map(|inner| Self(inner.pid))
    }
}

#[rustfmt::skip]
impl schemars::JsonSchema for StopGameParams {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "StopGameParams".into()
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        let mut properties = serde_json::Map::new();
        properties.insert(
            "pid".to_string(),
            generator.subschema_for::<u32>().as_value().clone(),
        );
        let mut object = serde_json::Map::new();
        object.insert("type".to_string(), "object".into());
        object.insert("properties".to_string(), properties.into());
        object.into()
    }
}
