use crate::Checker;
use crate::Diagnostic;
use crate::Project;
use atomic_delegate_macros::atomic_delegate;

pub struct TestInline;

impl TestInline {
    pub const NAME: &'static str = "test_inline";
    pub const CODE: &'static str = "E007";
}

#[rustfmt::skip]
impl Checker for TestInline {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    #[atomic_delegate(TestInline)]
    fn check(&self, project: &Project) -> Vec<Diagnostic> {}
}

#[rustfmt::skip]
impl TestInline {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {
        checkers.push(Box::new(TestInline));
    }
}

// no test_usage necessary
