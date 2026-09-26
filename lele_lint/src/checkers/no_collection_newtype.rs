use crate::Checker;
use crate::Diagnostic;
use crate::Project;
use atomic_delegate_macros::atomic_delegate;

pub struct NoCollectionNewtype;

impl NoCollectionNewtype {
    pub const NAME: &'static str = "no_collection_newtype";
    pub const CODE: &'static str = "E028";
}

#[rustfmt::skip]
impl Checker for NoCollectionNewtype {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    #[atomic_delegate(NoCollectionNewtype)]
    fn check(&self, project: &Project) -> Vec<Diagnostic> {}
}

#[rustfmt::skip]
impl NoCollectionNewtype {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {
        checkers.push(Box::new(NoCollectionNewtype));
    }
}

// no test_usage necessary
