use crate::checkers;
use crate::Checker;
use crate::Diagnostic;
use crate::Project;
use atomic_delegate_macros::atomic_delegates;

pub struct NoAllowAttributes;

impl NoAllowAttributes {
    pub const NAME: &'static str = "no_allow_attributes";
    pub const CODE: &'static str = "E023";
}

#[rustfmt::skip]
impl Checker for NoAllowAttributes {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn check(&self, project: &Project) -> Vec<Diagnostic> { checkers::no_allow_attributes_check::check(self, project) }
}

#[atomic_delegates]
impl NoAllowAttributes {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {}
}

// no test_usage necessary
