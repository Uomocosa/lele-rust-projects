use crate::checkers;
use crate::Checker;
use crate::Diagnostic;
use crate::Project;
use atomic_delegate_macros::atomic_delegates;

pub struct NoStutteredPath;

impl NoStutteredPath {
    pub const NAME: &'static str = "no_stuttered_path";
    pub const CODE: &'static str = "E025";
}

#[rustfmt::skip]
impl Checker for NoStutteredPath {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn check(&self, project: &Project) -> Vec<Diagnostic> { checkers::no_stuttered_path_check::check(self, project) }
}

#[atomic_delegates]
impl NoStutteredPath {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {}
}

// no test_usage necessary
