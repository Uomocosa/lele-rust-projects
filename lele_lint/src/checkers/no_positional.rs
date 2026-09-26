use crate::checkers;
use crate::Checker;
use crate::Diagnostic;
use crate::Project;
use atomic_delegate_macros::atomic_delegates;

pub struct NoPositional;

impl NoPositional {
    pub const NAME: &'static str = "no_positional";
    pub const CODE: &'static str = "E009";
}

#[rustfmt::skip]
impl Checker for NoPositional {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn check(&self, project: &Project) -> Vec<Diagnostic> { checkers::no_positional_check::check(self, project) }
}

#[atomic_delegates]
impl NoPositional {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {}
}

// no test_usage necessary
