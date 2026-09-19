use super::NamedButton;

#[must_use]
pub fn center_of(buttons: &[NamedButton], needle: &str) -> Option<(f64, f64)> {
    buttons
        .iter()
        .find(|button| button.name.contains(needle))
        .map(|button| (button.x, button.y))
}

#[cfg(test)]
mod tests {
    use super::NamedButton;
    use super::center_of;

    #[test]
    fn test_usage() {
        let buttons = vec![
            NamedButton {
                name: "create-new-room".to_string(),
                x: 40.0,
                y: 76.0,
            },
            NamedButton {
                name: "room:alpha".to_string(),
                x: 40.0,
                y: 120.0,
            },
        ];
        assert_eq!(center_of(&buttons, "create-new-room"), Some((40.0, 76.0)));
        assert_eq!(center_of(&buttons, "room:alpha"), Some((40.0, 120.0)));
        assert_eq!(center_of(&buttons, "missing"), None);
    }
}
