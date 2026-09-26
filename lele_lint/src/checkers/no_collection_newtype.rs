use crate::checkers;
use crate::Checker;
use crate::Diagnostic;
use crate::Project;
use atomic_delegate_macros::atomic_delegates;

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

#[atomic_delegates]
impl NoCollectionNewtype {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {}
}

// no test_usage necessary
