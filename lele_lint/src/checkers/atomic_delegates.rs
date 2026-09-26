use crate::Checker;
use crate::Diagnostic;
use crate::Project;
use atomic_delegate_macros::atomic_delegate;

pub struct AtomicDelegates;

impl AtomicDelegates {
    pub const NAME: &'static str = "atomic_delegates";
    pub const CODE: &'static str = "E012";
}

#[rustfmt::skip]
impl Checker for AtomicDelegates {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    #[atomic_delegate(AtomicDelegates)]
    fn check(&self, project: &Project) -> Vec<Diagnostic> {}
}

#[rustfmt::skip]
impl AtomicDelegates {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {
        checkers.push(Box::new(AtomicDelegates));
    }
}

// no test_usage necessary
