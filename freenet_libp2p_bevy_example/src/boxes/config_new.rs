use super::config::Config;
use crate::boxes;

pub fn new(own_id: boxes::PlayerId) -> Config {
    Config(own_id)
}

#[cfg(test)]
mod tests {
    use super::new;

    #[test]
    fn test_usage() {
        let config = new([7; 32]);
        assert_eq!(*config, [7; 32]);
    }
}
