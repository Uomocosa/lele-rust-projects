use serde_json::Value;

use super::NamedButton;

#[must_use]
pub fn parse_named_buttons(result: &Value) -> Vec<NamedButton> {
    let Some(rows) = result.as_array() else {
        return Vec::new();
    };
    let name_key = std::any::type_name::<bevy::prelude::Name>();
    let transform_key = std::any::type_name::<bevy::ui::UiGlobalTransform>();
    rows.iter()
        .filter_map(|row| parse_row(row, name_key, transform_key))
        .collect()
}

// needed helper: extracts one query row into a named button
fn parse_row(row: &Value, name_key: &str, transform_key: &str) -> Option<NamedButton> {
    let components = row.get("components")?.as_object()?;
    let name = component_string(components.get(name_key)?);
    let transform = components.get(transform_key)?.as_array()?;
    let x = transform.get(4)?.as_f64()?;
    let y = transform.get(5)?.as_f64()?;
    Some(NamedButton { name, x, y })
}

// needed helper: accepts either a plain string or a nested reflection shape
fn component_string(value: &Value) -> String {
    value
        .as_str()
        .map_or_else(|| value.to_string(), str::to_string)
}

#[cfg(test)]
mod tests {
    use super::parse_named_buttons;

    #[test]
    fn test_usage() {
        let name_key = std::any::type_name::<bevy::prelude::Name>();
        let transform_key = std::any::type_name::<bevy::ui::UiGlobalTransform>();
        let mut components = serde_json::Map::new();
        components.insert(name_key.to_string(), serde_json::json!("room:a"));
        components.insert(
            transform_key.to_string(),
            serde_json::json!([1.0, 0.0, 0.0, 1.0, 10.0, 20.0]),
        );
        let result = serde_json::json!([{ "entity": 1, "components": components }]);
        let buttons = parse_named_buttons(&result);
        assert_eq!(buttons.len(), 1);
        assert_eq!(buttons.first().map(|b| b.name.as_str()), Some("room:a"));
        assert_eq!(buttons.first().map(|b| (b.x, b.y)), Some((10.0, 20.0)));
    }
}
