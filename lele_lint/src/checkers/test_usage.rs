use crate::checkers;
use crate::Checker;
use crate::Diagnostic;
use crate::Project;
use atomic_delegate_macros::atomic_delegates;

pub struct TestUsage;

impl TestUsage {
    pub const NAME: &'static str = "test_usage";
    pub const CODE: &'static str = "E006";
}

#[rustfmt::skip]
impl Checker for TestUsage {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn check(&self, project: &Project) -> Vec<Diagnostic> { checkers::test_usage_check::check(self, project) }
}

#[atomic_delegates]
impl TestUsage {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {}
}

// no test_usage necessary
