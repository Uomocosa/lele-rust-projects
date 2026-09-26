use crate::checkers;
use crate::Checker;
use crate::Diagnostic;
use crate::Project;
use atomic_delegate_macros::atomic_delegates;

pub struct NoDunderTests;

impl NoDunderTests {
    pub const NAME: &'static str = "no_dunder_tests";
    pub const CODE: &'static str = "E034";
}

#[rustfmt::skip]
impl Checker for NoDunderTests {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn check(&self, project: &Project) -> Vec<Diagnostic> { checkers::no_dunder_tests_check::check(self, project) }
}

#[atomic_delegates]
impl NoDunderTests {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {}
}

// no test_usage necessary
