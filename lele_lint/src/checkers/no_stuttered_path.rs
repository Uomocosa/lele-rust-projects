use crate::Checker;
use crate::Diagnostic;
use crate::Project;
use atomic_delegate_macros::atomic_delegate;

pub struct NoStutteredPath;

impl NoStutteredPath {
    pub const NAME: &'static str = "no_stuttered_path";
    pub const CODE: &'static str = "E025";
}

#[rustfmt::skip]
impl Checker for NoStutteredPath {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    #[atomic_delegate(NoStutteredPath)]
    fn check(&self, project: &Project) -> Vec<Diagnostic> {}
}

#[rustfmt::skip]
impl NoStutteredPath {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {
        checkers.push(Box::new(NoStutteredPath));
    }
}

// no test_usage necessary
