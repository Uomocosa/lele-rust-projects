use super::config::Config;
use crate::boxes;

pub fn new(own_id: boxes::PlayerId) -> Config {
    Config(own_id)
}

#[cfg(test)]
mod tests {
    use super::new;
    use crate::boxes;

    #[test]
    fn test_usage() {
        let config = new(boxes::PlayerId(7));
        assert_eq!(*config, boxes::PlayerId(7));
    }
}
