use bevy::prelude::*;

#[derive(Component, Debug, Clone, Copy)]
pub struct CursorIcon;

#[cfg(test)]
mod tests {
    use super::CursorIcon;

    #[test]
    fn test_usage() {
        let icon = CursorIcon;
        let _ = format!("{icon:?}");
    }
}
