use serde_json::{Map, Value};

use crate::brp;

pub fn fixtures(schema: &Value, type_path: &str, max: usize) -> Vec<brp::Fixture> {
    let mut out: Vec<brp::Fixture> = Vec::new();
    let variants = candidates(schema, type_path, 0);
    for (position, variant) in variants.into_iter().enumerate() {
        let label = if position == 0 {
            "defaults".to_string()
        } else {
            variant.label
        };
        push_unique(&mut out, label, variant.value);
    }
    if let Some(value) = fullest(schema, type_path) {
        push_unique(&mut out, "every field at its fullest".to_string(), value);
    }
    out.truncate(max);
    out
}

// needed helper: skip fixtures whose value was already generated
fn push_unique(out: &mut Vec<brp::Fixture>, label: String, value: Value) {
    if !out.iter().any(|f| f.value == value) {
        out.push(brp::Fixture { label, value });
    }
}

// needed helper: every value to try for a type; the first entry is the base value
fn candidates(schema: &Value, type_path: &str, depth: usize) -> Vec<brp::Fixture> {
    let Some(def) = schema.get(type_path) else {
        return Vec::new();
    };
    if depth > brp::MAX_DEPTH {
        return Vec::new();
    }
    let next = depth.saturating_add(1);
    match def.get("kind").and_then(Value::as_str) {
        Some("Value") => value_candidates(type_path),
        Some("Struct") => struct_candidates(schema, def, next),
        Some("TupleStruct") => match def.get("prefixItems").and_then(Value::as_array) {
            Some(items) if items.len() == 1 => items
                .first()
                .and_then(ref_of)
                .map_or_else(Vec::new, |inner| candidates(schema, inner, next)),
            _ => Vec::new(),
        },
        Some("Enum") if type_path.starts_with("core::option::Option<") => {
            option_candidates(schema, def, next)
        }
        Some("Enum") => enum_candidates(schema, def, next),
        Some("Map") => map_candidates(schema, def, next),
        Some("List" | "Set") => list_candidates(schema, def, next),
        _ => Vec::new(),
    }
}

// needed helper: the `$defs` type path behind a `{"type": {"$ref": ...}}` node
fn ref_of(node: &Value) -> Option<&str> {
    node.get("type")?
        .get("$ref")?
        .as_str()?
        .strip_prefix("#/$defs/")
}

// needed helper: boundary values for primitive leaves
fn value_candidates(type_path: &str) -> Vec<brp::Fixture> {
    let pairs: Vec<(&str, Value)> = match type_path {
        "alloc::string::String" => {
            vec![("", Value::from("sample")), ("empty text", Value::from(""))]
        }
        "bool" => vec![("", Value::from(false)), ("true", Value::from(true))],
        "u8" | "u16" | "u32" | "u64" | "usize" => {
            vec![
                ("", Value::from(1)),
                ("0", Value::from(0)),
                ("max", max_of(type_path)),
            ]
        }
        "i8" | "i16" | "i32" | "i64" | "isize" => {
            vec![
                ("", Value::from(1)),
                ("0", Value::from(0)),
                ("-1", Value::from(-1)),
            ]
        }
        "f32" | "f64" => vec![("", Value::from(1.0)), ("0", Value::from(0.0))],
        _ => Vec::new(),
    };
    pairs
        .into_iter()
        .map(|(label, value)| brp::Fixture {
            label: label.to_string(),
            value,
        })
        .collect()
}

// needed helper: the maximum of an unsigned primitive
fn max_of(type_path: &str) -> Value {
    match type_path {
        "u8" => Value::from(u8::MAX),
        "u16" => Value::from(u16::MAX),
        "u32" => Value::from(u32::MAX),
        _ => Value::from(u64::MAX),
    }
}

// needed helper: base struct plus one-field-at-a-time variations, enums first
fn struct_candidates(schema: &Value, def: &Value, depth: usize) -> Vec<brp::Fixture> {
    let Some(properties) = def.get("properties").and_then(Value::as_object) else {
        return vec![brp::Fixture {
            label: String::new(),
            value: Value::Object(Map::new()),
        }];
    };
    let mut fields: Vec<(u8, &String, Vec<brp::Fixture>)> = Vec::new();
    for (name, node) in properties {
        let Some(field_type) = ref_of(node) else {
            return Vec::new();
        };
        let field_candidates = candidates(schema, field_type, depth);
        if field_candidates.is_empty() {
            return Vec::new();
        }
        fields.push((priority(schema, field_type), name, field_candidates));
    }
    fields.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(b.1)));
    let mut base = Map::new();
    for (_, name, field_candidates) in &fields {
        if let Some(first) = field_candidates.first() {
            base.insert((*name).clone(), first.value.clone());
        }
    }
    let mut out = vec![brp::Fixture {
        label: String::new(),
        value: Value::Object(base.clone()),
    }];
    for (_, name, field_candidates) in &fields {
        for variant in field_candidates.iter().skip(1) {
            let mut changed = base.clone();
            changed.insert((*name).clone(), variant.value.clone());
            out.push(brp::Fixture {
                label: format!("{name}: {}", variant.label),
                value: Value::Object(changed),
            });
        }
    }
    out
}

// needed helper: which fields to vary first (enums change UI the most)
fn priority(schema: &Value, type_path: &str) -> u8 {
    match schema
        .get(type_path)
        .and_then(|def| def.get("kind"))
        .and_then(Value::as_str)
    {
        Some("Enum") => 0,
        Some("Map" | "List" | "Set") => 1,
        Some("Struct" | "TupleStruct") => 2,
        _ => 3,
    }
}

// needed helper: None first, then every candidate of the inner type as Some
fn option_candidates(schema: &Value, def: &Value, depth: usize) -> Vec<brp::Fixture> {
    let inner = def
        .get("oneOf")
        .and_then(Value::as_array)
        .and_then(|variants| variants.get(1))
        .and_then(|some| some.get("prefixItems"))
        .and_then(Value::as_array)
        .and_then(|items| items.first())
        .and_then(ref_of);
    let mut out = vec![brp::Fixture {
        label: "None".to_string(),
        value: Value::Null,
    }];
    if let Some(inner) = inner {
        for variant in candidates(schema, inner, depth) {
            let label = if variant.label.is_empty() {
                "Some".to_string()
            } else {
                format!("Some > {}", variant.label)
            };
            out.push(brp::Fixture {
                label,
                value: variant.value,
            });
        }
    }
    out
}

// needed helper: one fixture per enum variant, payloads at their base value
fn enum_candidates(schema: &Value, def: &Value, depth: usize) -> Vec<brp::Fixture> {
    let Some(variants) = def.get("oneOf").and_then(Value::as_array) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for variant in variants {
        if let Some(name) = variant.as_str() {
            out.push(brp::Fixture {
                label: name.to_string(),
                value: Value::from(name),
            });
            continue;
        }
        let Some(name) = variant.get("shortPath").and_then(Value::as_str) else {
            continue;
        };
        let payload = variant
            .get("prefixItems")
            .and_then(Value::as_array)
            .and_then(|items| items.first())
            .and_then(ref_of)
            .and_then(|inner| candidates(schema, inner, depth).into_iter().next());
        let value = match payload {
            Some(inner) => {
                let mut wrapped = Map::new();
                wrapped.insert(name.to_string(), inner.value);
                Value::Object(wrapped)
            }
            None => Value::from(name),
        };
        out.push(brp::Fixture {
            label: name.to_string(),
            value,
        });
    }
    out
}

// needed helper: empty, one entry and several entries cycling the value variants
fn map_candidates(schema: &Value, def: &Value, depth: usize) -> Vec<brp::Fixture> {
    let empty = brp::Fixture {
        label: "empty".to_string(),
        value: Value::Object(Map::new()),
    };
    let Some(prefix) = def
        .get("keyType")
        .and_then(ref_of)
        .and_then(|key| string_key_prefix(schema, key))
    else {
        return vec![empty];
    };
    let values = def
        .get("valueType")
        .and_then(ref_of)
        .map_or_else(Vec::new, |value| candidates(schema, value, depth));
    if values.is_empty() {
        return vec![empty];
    }
    let mut out = vec![empty];
    for count in [1, brp::MANY] {
        let mut entries = Map::new();
        let mut used = Vec::new();
        for position in 0..count {
            let Some(variant) = position
                .checked_rem(values.len())
                .and_then(|i| values.get(i))
            else {
                continue;
            };
            entries.insert(
                format!("{prefix}_{}", position.saturating_add(1)),
                variant.value.clone(),
            );
            if !variant.label.is_empty() {
                used.push(variant.label.clone());
            }
        }
        let noun = if count == 1 { "entry" } else { "entries" };
        let label = if used.is_empty() {
            format!("{count} {noun}")
        } else {
            format!("{count} {noun} ({})", used.join(", "))
        };
        out.push(brp::Fixture {
            label,
            value: Value::Object(entries),
        });
    }
    out
}

// needed helper: empty, one item and several items cycling the item variants
fn list_candidates(schema: &Value, def: &Value, depth: usize) -> Vec<brp::Fixture> {
    let values = def
        .get("items")
        .and_then(ref_of)
        .map_or_else(Vec::new, |item| candidates(schema, item, depth));
    let mut out = vec![brp::Fixture {
        label: "empty".to_string(),
        value: Value::Array(Vec::new()),
    }];
    for count in [1, brp::MANY] {
        let items: Vec<Value> = (0..count)
            .filter_map(|position| {
                position
                    .checked_rem(values.len())
                    .and_then(|i| values.get(i))
            })
            .map(|variant| variant.value.clone())
            .collect();
        if !items.is_empty() {
            out.push(brp::Fixture {
                label: format!("{count} items"),
                value: Value::Array(items),
            });
        }
    }
    out
}

// needed helper: readable key prefix for string-keyed maps, e.g. RoomName -> "room"
fn string_key_prefix(schema: &Value, key_type: &str) -> Option<String> {
    let def = schema.get(key_type)?;
    let is_string = key_type == "alloc::string::String"
        || def
            .get("prefixItems")
            .and_then(Value::as_array)
            .and_then(|items| items.first())
            .and_then(ref_of)
            == Some("alloc::string::String");
    if !is_string {
        return None;
    }
    let short = def
        .get("shortPath")
        .and_then(Value::as_str)
        .unwrap_or("key");
    let mut snake = String::new();
    for (position, c) in short.chars().enumerate() {
        if c.is_ascii_uppercase() && position > 0 {
            snake.push('_');
        }
        snake.push(c.to_ascii_lowercase());
    }
    let trimmed = snake
        .strip_suffix("_name")
        .or_else(|| snake.strip_suffix("_id"))
        .unwrap_or(&snake);
    Some(trimmed.to_string())
}

// needed helper: a struct with every field set to its largest candidate
fn fullest(schema: &Value, type_path: &str) -> Option<Value> {
    let properties = schema.get(type_path)?.get("properties")?.as_object()?;
    let mut out = Map::new();
    for (name, node) in properties {
        let largest = candidates(schema, ref_of(node)?, 1)
            .into_iter()
            .max_by_key(|variant| variant.value.to_string().len())?;
        out.insert(name.clone(), largest.value);
    }
    Some(Value::Object(out))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::fixtures;

    #[test]
    fn test_usage() {
        let schema = json!({
            "demo::Lobby": {
                "shortPath": "Lobby", "kind": "Struct",
                "properties": {
                    "room": { "type": { "$ref": "#/$defs/core::option::Option<demo::Room>" } },
                    "rooms": { "type": { "$ref": "#/$defs/alloc::collections::BTreeMap<demo::RoomName, demo::Status>" } }
                }
            },
            "core::option::Option<demo::Room>": {
                "kind": "Enum",
                "oneOf": [
                    { "shortPath": "None" },
                    { "shortPath": "Some", "kind": "Tuple", "prefixItems": [ { "type": { "$ref": "#/$defs/demo::Room" } } ] }
                ]
            },
            "demo::Room": {
                "shortPath": "Room", "kind": "Struct",
                "properties": { "status": { "type": { "$ref": "#/$defs/demo::Status" } } }
            },
            "demo::Status": { "kind": "Enum", "oneOf": ["Known", "Connecting", "Connected"] },
            "demo::RoomName": {
                "shortPath": "RoomName", "kind": "TupleStruct",
                "prefixItems": [ { "type": { "$ref": "#/$defs/alloc::string::String" } } ]
            },
            "alloc::collections::BTreeMap<demo::RoomName, demo::Status>": {
                "kind": "Map",
                "keyType": { "type": { "$ref": "#/$defs/demo::RoomName" } },
                "valueType": { "type": { "$ref": "#/$defs/demo::Status" } }
            },
            "alloc::string::String": { "kind": "Value" }
        });
        let all = fixtures(&schema, "demo::Lobby", 50);
        let labels: Vec<&str> = all.iter().map(|f| f.label.as_str()).collect();
        assert_eq!(
            all.first().unwrap().value,
            json!({ "room": null, "rooms": {} })
        );
        assert!(labels.contains(&"room: Some > status: Connected"));
        assert!(labels.contains(&"rooms: 3 entries (Known, Connecting, Connected)"));
        assert!(labels.contains(&"every field at its fullest"));
        assert_eq!(
            all.iter()
                .find(|f| f.label.starts_with("rooms: 3"))
                .unwrap()
                .value["rooms"],
            json!({ "room_1": "Known", "room_2": "Connecting", "room_3": "Connected" })
        );
        assert_eq!(fixtures(&schema, "demo::Lobby", 2).len(), 2);
    }
}
