use crate::Checker;
use crate::Diagnostic;
use crate::Project;
use atomic_delegate_macros::atomic_delegate;

pub struct NoDunderTests;

impl NoDunderTests {
    pub const NAME: &'static str = "no_dunder_tests";
    pub const CODE: &'static str = "E034";
}

#[rustfmt::skip]
impl Checker for NoDunderTests {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    #[atomic_delegate(NoDunderTests)]
    fn check(&self, project: &Project) -> Vec<Diagnostic> {}
}

#[rustfmt::skip]
impl NoDunderTests {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {
        checkers.push(Box::new(NoDunderTests));
    }
}

// no test_usage necessary
