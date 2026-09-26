use crate::checkers;
use crate::Checker;
use crate::Diagnostic;
use crate::Project;
use atomic_delegate_macros::atomic_delegates;

pub struct ConstructorNoSkip;

impl ConstructorNoSkip {
    pub const NAME: &'static str = "constructor_no_skip";
    pub const CODE: &'static str = "E013";
}

#[rustfmt::skip]
impl Checker for ConstructorNoSkip {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn check(&self, project: &Project) -> Vec<Diagnostic> { checkers::constructor_no_skip_check::check(self, project) }
}

#[atomic_delegates]
impl ConstructorNoSkip {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {}
}

// no test_usage necessary
