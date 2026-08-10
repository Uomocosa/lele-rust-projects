use bevy::prelude::Component;

#[derive(Component)]
pub struct CounterText;

#[cfg(test)]
mod tests {
    use super::CounterText;

    #[test]
    fn test_usage() {
        let _text = CounterText;
    }
}
