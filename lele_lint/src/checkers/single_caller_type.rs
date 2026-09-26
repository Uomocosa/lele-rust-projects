use crate::Checker;
use crate::Diagnostic;
use crate::Project;
use atomic_delegate_macros::atomic_delegate;

pub struct SingleCallerType;

impl SingleCallerType {
    pub const NAME: &'static str = "single_caller_type";
    pub const CODE: &'static str = "E016";
}

#[rustfmt::skip]
impl Checker for SingleCallerType {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    #[atomic_delegate(SingleCallerType)]
    fn check(&self, project: &Project) -> Vec<Diagnostic> {}
}

#[rustfmt::skip]
impl SingleCallerType {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {
        checkers.push(Box::new(SingleCallerType));
    }
}

// no test_usage necessary
