use crate::Checker;
use crate::Diagnostic;
use crate::Project;
use atomic_delegate_macros::atomic_delegate;

pub struct NoSuperImports;

impl NoSuperImports {
    pub const NAME: &'static str = "no_super_imports";
    pub const CODE: &'static str = "E033";
}

#[rustfmt::skip]
impl Checker for NoSuperImports {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    #[atomic_delegate(NoSuperImports)]
    fn check(&self, project: &Project) -> Vec<Diagnostic> {}
}

#[rustfmt::skip]
impl NoSuperImports {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {
        checkers.push(Box::new(NoSuperImports));
    }
}

// no test_usage necessary
