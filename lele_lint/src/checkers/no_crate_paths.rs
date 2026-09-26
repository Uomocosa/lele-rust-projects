use crate::Checker;
use crate::Diagnostic;
use crate::Project;
use atomic_delegate_macros::atomic_delegate;

pub struct NoCratePaths;

impl NoCratePaths {
    pub const NAME: &'static str = "no_crate_paths";
    pub const CODE: &'static str = "E020";
}

#[rustfmt::skip]
impl Checker for NoCratePaths {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    #[atomic_delegate(NoCratePaths)]
    fn check(&self, project: &Project) -> Vec<Diagnostic> {}
}

#[rustfmt::skip]
impl NoCratePaths {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {
        checkers.push(Box::new(NoCratePaths));
    }
}

// no test_usage necessary
