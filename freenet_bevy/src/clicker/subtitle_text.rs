use bevy::prelude::Component;

#[derive(Component)]
pub struct SubtitleText;

#[cfg(test)]
mod tests {
    use super::SubtitleText;

    #[test]
    fn test_usage() {
        let _text = SubtitleText;
    }
}
