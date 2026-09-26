use crate::checkers;
use crate::Checker;
use crate::Config;
use crate::Diagnostic;
use crate::Project;

pub struct NoCollectionNewtype;

impl NoCollectionNewtype {
    pub const NAME: &'static str = "no_collection_newtype";
    pub const CODE: &'static str = "E028";
}

#[rustfmt::skip]
impl Checker for NoCollectionNewtype {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn check(&self, project: &Project) -> Vec<Diagnostic> { checkers::no_collection_newtype_check::check(self, project) }
}

#[rustfmt::skip]
impl NoCollectionNewtype {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
        checkers::no_collection_newtype_register::register(checkers, config)
    }
}

// no test_usage necessary
