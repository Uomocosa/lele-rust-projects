use crate::checkers;
use crate::Checker;

pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {
    checkers.push(Box::new(checkers::no_crate_paths::NoCratePaths));
}

#[cfg(test)]
mod tests {
    use super::register;

    #[test]
    fn test_usage() {
        let mut checkers = Vec::new();
        register(&mut checkers);
        assert_eq!(checkers.len(), 1);
        assert_eq!(checkers[0].code(), "E020");
    }
}
