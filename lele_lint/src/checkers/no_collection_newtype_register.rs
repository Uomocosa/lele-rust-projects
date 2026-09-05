use super::no_collection_newtype::NoCollectionNewtype;
use crate::Checker;
use crate::Config;

pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
    if config.checker_enabled("no_collection_newtype") {
        checkers.push(Box::new(NoCollectionNewtype));
    }
}

// no test_usage necessary
