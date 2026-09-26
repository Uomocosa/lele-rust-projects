use crate::Checker;
use crate::Diagnostic;
use crate::Project;
use atomic_delegate_macros::atomic_delegate;

pub struct DelegateMacro;

impl DelegateMacro {
    pub const NAME: &'static str = "delegate_macro";
    pub const CODE: &'static str = "E032";
}

#[rustfmt::skip]
impl Checker for DelegateMacro {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    #[atomic_delegate(DelegateMacro)]
    fn check(&self, project: &Project) -> Vec<Diagnostic> {}
}

#[rustfmt::skip]
impl DelegateMacro {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {
        checkers.push(Box::new(DelegateMacro));
    }
}

// no test_usage necessary
