use crate::checkers;
use crate::Checker;
use crate::Diagnostic;
use crate::Project;
use atomic_delegate_macros::atomic_delegates;

pub struct NoSuperImports;

impl NoSuperImports {
    pub const NAME: &'static str = "no_super_imports";
    pub const CODE: &'static str = "E033";
}

#[rustfmt::skip]
impl Checker for NoSuperImports {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn check(&self, project: &Project) -> Vec<Diagnostic> { checkers::no_super_imports_check::check(self, project) }
}

#[atomic_delegates]
impl NoSuperImports {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {}
}

// no test_usage necessary
