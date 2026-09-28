use crate::web;

pub fn action_label(action: &web::Action) -> String {
    match action {
        web::Action::Navigate { label, .. } => format!("open '{label}'"),
        web::Action::Click { label, .. } => format!("click '{label}'"),
        web::Action::Fill { text, label, .. } => format!("type '{text}' in '{label}'"),
        web::Action::Key { key } => format!("press {key}"),
        web::Action::ScrollBottom => "scroll to bottom".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::action_label;
    use crate::web;

    #[test]
    fn test_usage() {
        let key = web::Action::Key {
            key: "Escape".to_string(),
        };
        assert_eq!(action_label(&key), "press Escape");
        assert_eq!(action_label(&web::Action::ScrollBottom), "scroll to bottom");
    }
}
