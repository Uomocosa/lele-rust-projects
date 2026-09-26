use crate::checkers;
use crate::Checker;
use crate::Diagnostic;
use crate::Project;
use atomic_delegate_macros::atomic_delegates;

pub struct SingleCallerType;

impl SingleCallerType {
    pub const NAME: &'static str = "single_caller_type";
    pub const CODE: &'static str = "E016";
}

#[rustfmt::skip]
impl Checker for SingleCallerType {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn check(&self, project: &Project) -> Vec<Diagnostic> { checkers::single_caller_type_check::check(self, project) }
}

#[atomic_delegates]
impl SingleCallerType {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {}
}

// no test_usage necessary
