use crate::Checker;
use crate::Diagnostic;
use crate::Project;
use atomic_delegate_macros::atomic_delegate;

pub struct NoAllowAttributes;

impl NoAllowAttributes {
    pub const NAME: &'static str = "no_allow_attributes";
    pub const CODE: &'static str = "E023";
}

#[rustfmt::skip]
impl Checker for NoAllowAttributes {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    #[atomic_delegate(NoAllowAttributes)]
    fn check(&self, project: &Project) -> Vec<Diagnostic> {}
}

#[rustfmt::skip]
impl NoAllowAttributes {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {
        checkers.push(Box::new(NoAllowAttributes));
    }
}

// no test_usage necessary
