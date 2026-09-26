use crate::Checker;
use crate::Diagnostic;
use crate::Project;
use atomic_delegate_macros::atomic_delegate;

pub struct ConstructorNoSkip;

impl ConstructorNoSkip {
    pub const NAME: &'static str = "constructor_no_skip";
    pub const CODE: &'static str = "E013";
}

#[rustfmt::skip]
impl Checker for ConstructorNoSkip {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    #[atomic_delegate(ConstructorNoSkip)]
    fn check(&self, project: &Project) -> Vec<Diagnostic> {}
}

#[rustfmt::skip]
impl ConstructorNoSkip {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {
        checkers.push(Box::new(ConstructorNoSkip));
    }
}

// no test_usage necessary
