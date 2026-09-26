use crate::Checker;
use crate::Diagnostic;
use crate::Project;
use atomic_delegate_macros::atomic_delegate;

pub struct NoTrivialAccessors;

impl NoTrivialAccessors {
    pub const NAME: &'static str = "no_trivial_accessors";
    pub const CODE: &'static str = "E010";
}

#[rustfmt::skip]
impl Checker for NoTrivialAccessors {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    #[atomic_delegate(NoTrivialAccessors)]
    fn check(&self, project: &Project) -> Vec<Diagnostic> {}
}

#[rustfmt::skip]
impl NoTrivialAccessors {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {
        checkers.push(Box::new(NoTrivialAccessors));
    }
}

// no test_usage necessary
